use crate::wallet::error::WalletError;
use crate::wallet::format::print_success;
use std::fs;
use std::path::Path;

pub fn run(name: &str) -> Result<(), WalletError> {
    // Validate name (lowercase alphanumeric + hyphens).
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(WalletError::Other(
            "loom name must be lowercase alphanumeric with hyphens".to_string(),
        ));
    }

    let dir = Path::new(name);
    if dir.exists() {
        return Err(WalletError::Other(format!(
            "directory '{}' already exists",
            name
        )));
    }

    // Create directory structure.
    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join(".cargo"))?;

    // Cargo.toml
    let crate_name = name.replace('-', "_");
    let cargo_toml = format!(
        r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
norn-sdk = {{ git = "https://github.com/augmnt/norn-protocol", tag = "v0.13.0" }}
borsh = {{ version = "1.5", default-features = false, features = ["derive"] }}

[profile.release]
opt-level = "z"
lto = true
strip = true
"#
    );
    fs::write(dir.join("Cargo.toml"), cargo_toml)?;

    // .cargo/config.toml — default to wasm32 target.
    let cargo_config = r#"[build]
target = "wasm32-unknown-unknown"
"#;
    fs::write(dir.join(".cargo/config.toml"), cargo_config)?;

    // src/lib.rs — SDK v3 contract template with Item, Response, ensure!, and a test.
    let lib_rs = format!(
        r#"//! {} — a Norn loom smart contract.

#![no_std]

extern crate alloc;

use norn_sdk::prelude::*;

// ── Storage ────────────────────────────────────────────────────────────────

const OWNER: Item<Address> = Item::new("owner");
const VALUE: Item<u64> = Item::new("value");

// ── Contract ───────────────────────────────────────────────────────────────

#[derive(BorshSerialize, BorshDeserialize)]
pub struct MyContract;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Execute {{
    SetValue {{ value: u64 }},
}}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Query {{
    GetValue,
}}

impl Contract for MyContract {{
    type Init = Empty;
    type Exec = Execute;
    type Query = Query;

    fn init(ctx: &Context, _msg: Empty) -> Self {{
        OWNER.save(&ctx.sender()).unwrap();
        VALUE.save(&0u64).unwrap();
        MyContract
    }}

    fn execute(&mut self, ctx: &Context, msg: Execute) -> ContractResult {{
        match msg {{
            Execute::SetValue {{ value }} => {{
                let owner = OWNER.load()?;
                ctx.require_sender(&owner)?;
                VALUE.save(&value)?;
                Ok(Response::new()
                    .add_attribute("action", "set_value")
                    .add_attribute("value", format!("{{value}}"))
                    .set_data(&value))
            }}
        }}
    }}

    fn query(&self, _ctx: &Context, msg: Query) -> ContractResult {{
        match msg {{
            Query::GetValue => ok(VALUE.load_or(0u64)),
        }}
    }}
}}

norn_entry!(MyContract);

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {{
    use super::*;
    use norn_sdk::testing::*;

    const ALICE: Address = [1u8; 20];

    #[test]
    fn test_set_and_get() {{
        let env = TestEnv::new().with_sender(ALICE);
        let mut contract = MyContract::init(&env.ctx(), Empty);

        let resp = contract
            .execute(&env.ctx(), Execute::SetValue {{ value: 42 }})
            .unwrap();
        assert_attribute(&resp, "action", "set_value");

        let resp = contract.query(&env.ctx(), Query::GetValue).unwrap();
        let val: u64 = from_response(&resp).unwrap();
        assert_eq!(val, 42);
    }}

    #[test]
    fn test_unauthorized() {{
        let env = TestEnv::new().with_sender(ALICE);
        let mut contract = MyContract::init(&env.ctx(), Empty);

        env.set_sender([2u8; 20]); // different sender
        let err = contract
            .execute(&env.ctx(), Execute::SetValue {{ value: 99 }})
            .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }}
}}
"#,
        name
    );
    fs::write(dir.join("src/lib.rs"), lib_rs)?;

    println!();
    print_success(&format!("Created loom project '{}'", name));
    println!();
    println!("  Build:");
    println!("    cd {name}");
    println!("    cargo build --release");
    println!();
    println!("  Test:");
    println!("    cargo test --target x86_64-apple-darwin");
    println!();
    println!("  Deploy:");
    println!("    norn wallet deploy-loom --name \"{name}\" --yes");
    println!("    norn wallet upload-bytecode --loom-id <ID> \\");
    println!("      --bytecode target/wasm32-unknown-unknown/release/{crate_name}.wasm");
    println!();

    Ok(())
}
