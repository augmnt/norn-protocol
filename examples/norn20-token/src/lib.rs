//! Norn20Token — A fungible token built entirely with the SDK standard library.
//!
//! Demonstrates how `Ownable`, `Pausable`, and `Norn20` compose to produce a
//! fully-featured ERC20-equivalent in ~60 lines of application code.

#![no_std]

extern crate alloc;

use norn_sdk::prelude::*;

// ── Contract ─────────────────────────────────────────────────────────────────

/// Unit struct — all state lives in the stdlib storage modules.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Norn20Token;

/// Constructor parameters.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct InitMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_supply: u128,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Execute {
    Transfer { to: Address, amount: u128 },
    Approve { spender: Address, amount: u128 },
    TransferFrom { from: Address, to: Address, amount: u128 },
    Mint { to: Address, amount: u128 },
    Burn { from: Address, amount: u128 },
    TransferOwnership { new_owner: Address },
    Pause,
    Unpause,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Query {
    Balance { addr: Address },
    Allowance { owner: Address, spender: Address },
    TotalSupply,
    Info,
    Owner,
    IsPaused,
}

impl Contract for Norn20Token {
    type Init = InitMsg;
    type Exec = Execute;
    type Query = Query;

    fn init(ctx: &Context, msg: InitMsg) -> Self {
        Ownable::init(&ctx.sender()).unwrap();
        Pausable::init().unwrap();
        Norn20::init(&msg.name, &msg.symbol, msg.decimals).unwrap();
        if msg.initial_supply > 0 {
            Norn20::mint(&ctx.sender(), msg.initial_supply).unwrap();
        }
        Norn20Token
    }

    fn execute(&mut self, ctx: &Context, msg: Execute) -> ContractResult {
        match msg {
            Execute::Transfer { to, amount } => {
                Pausable::require_not_paused()?;
                Norn20::transfer(ctx, &to, amount)
            }
            Execute::Approve { spender, amount } => {
                Pausable::require_not_paused()?;
                Norn20::approve(ctx, &spender, amount)
            }
            Execute::TransferFrom { from, to, amount } => {
                Pausable::require_not_paused()?;
                Norn20::transfer_from(ctx, &from, &to, amount)
            }
            Execute::Mint { to, amount } => {
                Ownable::require_owner(ctx)?;
                Norn20::mint(&to, amount)
            }
            Execute::Burn { from, amount } => {
                Ownable::require_owner(ctx)?;
                Norn20::burn(&from, amount)
            }
            Execute::TransferOwnership { new_owner } => {
                Ownable::transfer_ownership(ctx, &new_owner)
            }
            Execute::Pause => Pausable::pause(ctx),
            Execute::Unpause => Pausable::unpause(ctx),
        }
    }

    fn query(&self, _ctx: &Context, msg: Query) -> ContractResult {
        match msg {
            Query::Balance { addr } => ok(Norn20::balance_of(&addr)),
            Query::Allowance { owner, spender } => ok(Norn20::allowance(&owner, &spender)),
            Query::TotalSupply => ok(Norn20::total_supply()),
            Query::Info => ok(Norn20::info()?),
            Query::Owner => ok(Ownable::owner()?),
            Query::IsPaused => ok(Pausable::is_paused()),
        }
    }
}

norn_entry!(Norn20Token);

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    const ALICE: Address = [1u8; 20];
    const BOB: Address = [2u8; 20];
    const CHARLIE: Address = [3u8; 20];

    fn setup() -> (TestEnv, Norn20Token) {
        let env = TestEnv::new().with_sender(ALICE);
        let token = Norn20Token::init(
            &env.ctx(),
            InitMsg {
                name: String::from("Test Token"),
                symbol: String::from("TEST"),
                decimals: 18,
                initial_supply: 1_000_000,
            },
        );
        (env, token)
    }

    #[test]
    fn test_init() {
        let (env, token) = setup();
        let resp = token.query(&env.ctx(), Query::Info).unwrap();
        let info: Norn20Info = from_response(&resp).unwrap();
        assert_eq!(info.name, "Test Token");
        assert_eq!(info.symbol, "TEST");
        assert_eq!(info.decimals, 18);
        assert_eq!(info.total_supply, 1_000_000);
        assert_eq!(Norn20::balance_of(&ALICE), 1_000_000);
    }

    #[test]
    fn test_owner() {
        let (env, token) = setup();
        let resp = token.query(&env.ctx(), Query::Owner).unwrap();
        let owner: Address = from_response(&resp).unwrap();
        assert_eq!(owner, ALICE);
    }

    #[test]
    fn test_transfer() {
        let (env, mut token) = setup();
        let resp = token
            .execute(&env.ctx(), Execute::Transfer { to: BOB, amount: 1000 })
            .unwrap();
        assert_event(&resp, "Transfer");
        assert_eq!(Norn20::balance_of(&ALICE), 999_000);
        assert_eq!(Norn20::balance_of(&BOB), 1000);
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let (env, mut token) = setup();
        token
            .execute(&env.ctx(), Execute::Approve { spender: BOB, amount: 500 })
            .unwrap();
        assert_eq!(Norn20::allowance(&ALICE, &BOB), 500);

        env.set_sender(BOB);
        let resp = token
            .execute(
                &env.ctx(),
                Execute::TransferFrom {
                    from: ALICE,
                    to: CHARLIE,
                    amount: 200,
                },
            )
            .unwrap();
        assert_event(&resp, "Transfer");
        assert_eq!(Norn20::balance_of(&CHARLIE), 200);
        assert_eq!(Norn20::allowance(&ALICE, &BOB), 300);
    }

    #[test]
    fn test_mint_owner_only() {
        let (env, mut token) = setup();
        let resp = token
            .execute(&env.ctx(), Execute::Mint { to: BOB, amount: 5000 })
            .unwrap();
        assert_event(&resp, "Mint");
        assert_eq!(Norn20::balance_of(&BOB), 5000);

        // Non-owner can't mint
        env.set_sender(BOB);
        let err = token
            .execute(&env.ctx(), Execute::Mint { to: BOB, amount: 100 })
            .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_burn_owner_only() {
        let (env, mut token) = setup();
        let resp = token
            .execute(&env.ctx(), Execute::Burn { from: ALICE, amount: 500 })
            .unwrap();
        assert_event(&resp, "Burn");
        assert_eq!(Norn20::balance_of(&ALICE), 999_500);

        env.set_sender(BOB);
        let err = token
            .execute(&env.ctx(), Execute::Burn { from: ALICE, amount: 1 })
            .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_pause_blocks_transfers() {
        let (env, mut token) = setup();
        token.execute(&env.ctx(), Execute::Pause).unwrap();

        let err = token
            .execute(&env.ctx(), Execute::Transfer { to: BOB, amount: 10 })
            .unwrap_err();
        assert_eq!(err.message(), "contract is paused");

        // Unpause and retry
        token.execute(&env.ctx(), Execute::Unpause).unwrap();
        token
            .execute(&env.ctx(), Execute::Transfer { to: BOB, amount: 10 })
            .unwrap();
        assert_eq!(Norn20::balance_of(&BOB), 10);
    }

    #[test]
    fn test_pause_unauthorized() {
        let (env, mut token) = setup();
        env.set_sender(BOB);
        let err = token.execute(&env.ctx(), Execute::Pause).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_transfer_ownership() {
        let (env, mut token) = setup();
        let resp = token
            .execute(&env.ctx(), Execute::TransferOwnership { new_owner: BOB })
            .unwrap();
        assert_event(&resp, "OwnershipTransferred");
        assert_eq!(Ownable::owner().unwrap(), BOB);

        // Alice can no longer mint
        let err = token
            .execute(&env.ctx(), Execute::Mint { to: ALICE, amount: 1 })
            .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));

        // Bob can now mint
        env.set_sender(BOB);
        token
            .execute(&env.ctx(), Execute::Mint { to: BOB, amount: 100 })
            .unwrap();
    }

    #[test]
    fn test_query_is_paused() {
        let (env, mut token) = setup();
        let resp = token.query(&env.ctx(), Query::IsPaused).unwrap();
        let paused: bool = from_response(&resp).unwrap();
        assert!(!paused);

        token.execute(&env.ctx(), Execute::Pause).unwrap();
        let resp = token.query(&env.ctx(), Query::IsPaused).unwrap();
        let paused: bool = from_response(&resp).unwrap();
        assert!(paused);
    }

    #[test]
    fn test_mint_does_not_require_unpause() {
        let (env, mut token) = setup();
        token.execute(&env.ctx(), Execute::Pause).unwrap();
        // Minting is not gated by pause
        token
            .execute(&env.ctx(), Execute::Mint { to: BOB, amount: 100 })
            .unwrap();
        assert_eq!(Norn20::balance_of(&BOB), 100);
    }
}
