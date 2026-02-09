//! Token vault contract — demonstrates SDK v3 storage primitives, Response
//! builder, guard macros, and native testing.
//!
//! The vault has an owner who can deposit, withdraw tokens, and rename it.
//! State lives in `Item`/`Map` storage, not in the contract struct.

#![no_std]

extern crate alloc;

use norn_sdk::prelude::*;

// ── Storage layout ─────────────────────────────────────────────────────────

const OWNER: Item<Address> = Item::new("owner");
const NAME: Item<String> = Item::new("name");
const BALANCE: Item<u128> = Item::new("balance");
const TOKEN_ID: Item<TokenId> = Item::new("token_id");

// ── Contract ───────────────────────────────────────────────────────────────

/// Unit struct — all state lives in `Item` storage.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct TokenVault;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Execute {
    Deposit { amount: u128 },
    Withdraw { to: Address, amount: u128 },
    SetName { name: String },
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Query {
    GetInfo,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VaultInfo {
    pub owner: Address,
    pub name: String,
    pub balance: u128,
    pub token_id: TokenId,
}

impl Contract for TokenVault {
    type Init = Empty;
    type Exec = Execute;
    type Query = Query;

    fn init(ctx: &Context, _msg: Empty) -> Self {
        OWNER.save(&ctx.sender()).unwrap();
        NAME.save(&String::from("vault")).unwrap();
        BALANCE.save(&0u128).unwrap();
        TOKEN_ID.save(&[0u8; 32]).unwrap();
        TokenVault
    }

    fn execute(&mut self, ctx: &Context, msg: Execute) -> ContractResult {
        match msg {
            Execute::Deposit { amount } => {
                ensure!(amount > 0, "deposit amount must be positive");
                let bal = BALANCE.load_or(0u128);
                let new_bal = bal.checked_add(amount).ok_or(ContractError::Overflow)?;
                BALANCE.save(&new_bal)?;
                Ok(Response::new()
                    .add_attribute("action", "deposit")
                    .add_attribute("amount", format!("{amount}"))
                    .set_data(&new_bal))
            }
            Execute::Withdraw { to, amount } => {
                let owner = OWNER.load()?;
                ctx.require_sender(&owner)?;
                let bal = BALANCE.load_or(0u128);
                ensure!(amount <= bal, ContractError::InsufficientFunds);
                let new_bal = bal - amount;
                BALANCE.save(&new_bal)?;
                let token = TOKEN_ID.load_or([0u8; 32]);
                ctx.transfer(&owner, &to, &token, amount);
                Ok(Response::new()
                    .add_attribute("action", "withdraw")
                    .add_attribute("amount", format!("{amount}"))
                    .set_data(&new_bal))
            }
            Execute::SetName { name } => {
                let owner = OWNER.load()?;
                ctx.require_sender(&owner)?;
                NAME.save(&name)?;
                Ok(Response::new()
                    .add_attribute("action", "set_name")
                    .add_attribute("name", name))
            }
        }
    }

    fn query(&self, _ctx: &Context, msg: Query) -> ContractResult {
        match msg {
            Query::GetInfo => ok(VaultInfo {
                owner: OWNER.load_or(ZERO_ADDRESS),
                name: NAME.load_or(String::from("")),
                balance: BALANCE.load_or(0u128),
                token_id: TOKEN_ID.load_or([0u8; 32]),
            }),
        }
    }
}

norn_entry!(TokenVault);

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    const ALICE: Address = [1u8; 20];
    const BOB: Address = [2u8; 20];

    #[test]
    fn test_init_sets_owner() {
        let env = TestEnv::new().with_sender(ALICE);
        TokenVault::init(&env.ctx(), Empty);
        assert_eq!(OWNER.load().unwrap(), ALICE);
        assert_eq!(BALANCE.load().unwrap(), 0);
    }

    #[test]
    fn test_deposit() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut vault = TokenVault::init(&env.ctx(), Empty);
        let resp = vault
            .execute(&env.ctx(), Execute::Deposit { amount: 500 })
            .unwrap();
        assert_attribute(&resp, "action", "deposit");
        assert_attribute(&resp, "amount", "500");
        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 500);
    }

    #[test]
    fn test_deposit_zero_fails() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut vault = TokenVault::init(&env.ctx(), Empty);
        let err = vault
            .execute(&env.ctx(), Execute::Deposit { amount: 0 })
            .unwrap_err();
        assert_eq!(err.message(), "deposit amount must be positive");
    }

    #[test]
    fn test_withdraw_owner_only() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut vault = TokenVault::init(&env.ctx(), Empty);
        vault
            .execute(&env.ctx(), Execute::Deposit { amount: 100 })
            .unwrap();

        // Bob tries to withdraw
        env.set_sender(BOB);
        let err = vault
            .execute(
                &env.ctx(),
                Execute::Withdraw {
                    to: BOB,
                    amount: 50,
                },
            )
            .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));

        // Alice withdraws
        env.set_sender(ALICE);
        let resp = vault
            .execute(
                &env.ctx(),
                Execute::Withdraw {
                    to: BOB,
                    amount: 50,
                },
            )
            .unwrap();
        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 50);
    }

    #[test]
    fn test_withdraw_insufficient() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut vault = TokenVault::init(&env.ctx(), Empty);
        vault
            .execute(&env.ctx(), Execute::Deposit { amount: 10 })
            .unwrap();
        let err = vault
            .execute(
                &env.ctx(),
                Execute::Withdraw {
                    to: BOB,
                    amount: 100,
                },
            )
            .unwrap_err();
        assert!(matches!(err, ContractError::InsufficientFunds));
    }

    #[test]
    fn test_set_name() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut vault = TokenVault::init(&env.ctx(), Empty);
        let resp = vault
            .execute(
                &env.ctx(),
                Execute::SetName {
                    name: String::from("my-vault"),
                },
            )
            .unwrap();
        assert_attribute(&resp, "action", "set_name");
        assert_attribute(&resp, "name", "my-vault");
        assert_eq!(NAME.load().unwrap(), "my-vault");
    }

    #[test]
    fn test_query_info() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut vault = TokenVault::init(&env.ctx(), Empty);
        vault
            .execute(&env.ctx(), Execute::Deposit { amount: 42 })
            .unwrap();
        let resp = vault.query(&env.ctx(), Query::GetInfo).unwrap();
        let info: VaultInfo = from_response(&resp).unwrap();
        assert_eq!(info.owner, ALICE);
        assert_eq!(info.balance, 42);
        assert_eq!(info.name, "vault");
    }
}
