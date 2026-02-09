//! Token vault contract — demonstrates `#[norn_contract]` with `Item` storage,
//! `Response` builder, guard macros, and native testing.

#![no_std]

extern crate alloc;

use norn_sdk::prelude::*;

// ── Storage layout ─────────────────────────────────────────────────────────

const OWNER: Item<Address> = Item::new("owner");
const NAME: Item<String> = Item::new("name");
const BALANCE: Item<u128> = Item::new("balance");
const TOKEN_ID: Item<TokenId> = Item::new("token_id");

// ── Contract ───────────────────────────────────────────────────────────────

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VaultInfo {
    pub owner: Address,
    pub name: String,
    pub balance: u128,
    pub token_id: TokenId,
}

#[norn_contract]
pub struct TokenVault;

#[norn_contract]
impl TokenVault {
    #[init]
    pub fn new(ctx: &Context) -> Self {
        OWNER.save(&ctx.sender()).unwrap();
        NAME.save(&String::from("vault")).unwrap();
        BALANCE.save(&0u128).unwrap();
        TOKEN_ID.save(&[0u8; 32]).unwrap();
        TokenVault
    }

    #[execute]
    pub fn deposit(&mut self, _ctx: &Context, amount: u128) -> ContractResult {
        ensure!(amount > 0, "deposit amount must be positive");
        let bal = BALANCE.load_or(0u128);
        let new_bal = bal.checked_add(amount).ok_or(ContractError::Overflow)?;
        BALANCE.save(&new_bal)?;
        Ok(Response::new()
            .add_attribute("action", "deposit")
            .add_attribute("amount", format!("{amount}"))
            .set_data(&new_bal))
    }

    #[execute]
    pub fn withdraw(&mut self, ctx: &Context, to: Address, amount: u128) -> ContractResult {
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

    #[execute]
    pub fn set_name(&mut self, ctx: &Context, name: String) -> ContractResult {
        let owner = OWNER.load()?;
        ctx.require_sender(&owner)?;
        NAME.save(&name)?;
        Ok(Response::new()
            .add_attribute("action", "set_name")
            .add_attribute("name", name))
    }

    #[query]
    pub fn get_info(&self, _ctx: &Context) -> ContractResult {
        ok(VaultInfo {
            owner: OWNER.load_or(ZERO_ADDRESS),
            name: NAME.load_or(String::from("")),
            balance: BALANCE.load_or(0u128),
            token_id: TOKEN_ID.load_or([0u8; 32]),
        })
    }
}

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
        TokenVault::new(&env.ctx());
        assert_eq!(OWNER.load().unwrap(), ALICE);
        assert_eq!(BALANCE.load().unwrap(), 0);
    }

    #[test]
    fn test_deposit() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut vault = TokenVault::new(&env.ctx());
        let resp = vault.deposit(&env.ctx(), 500).unwrap();
        assert_attribute(&resp, "action", "deposit");
        assert_attribute(&resp, "amount", "500");
        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 500);
    }

    #[test]
    fn test_deposit_zero_fails() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut vault = TokenVault::new(&env.ctx());
        let err = vault.deposit(&env.ctx(), 0).unwrap_err();
        assert_eq!(err.message(), "deposit amount must be positive");
    }

    #[test]
    fn test_withdraw_owner_only() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut vault = TokenVault::new(&env.ctx());
        vault.deposit(&env.ctx(), 100).unwrap();

        // Bob tries to withdraw
        env.set_sender(BOB);
        let err = vault.withdraw(&env.ctx(), BOB, 50).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));

        // Alice withdraws
        env.set_sender(ALICE);
        let resp = vault.withdraw(&env.ctx(), BOB, 50).unwrap();
        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 50);
    }

    #[test]
    fn test_withdraw_insufficient() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut vault = TokenVault::new(&env.ctx());
        vault.deposit(&env.ctx(), 10).unwrap();
        let err = vault.withdraw(&env.ctx(), BOB, 100).unwrap_err();
        assert!(matches!(err, ContractError::InsufficientFunds));
    }

    #[test]
    fn test_set_name() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut vault = TokenVault::new(&env.ctx());
        let resp = vault.set_name(&env.ctx(), String::from("my-vault")).unwrap();
        assert_attribute(&resp, "action", "set_name");
        assert_attribute(&resp, "name", "my-vault");
        assert_eq!(NAME.load().unwrap(), "my-vault");
    }

    #[test]
    fn test_query_info() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut vault = TokenVault::new(&env.ctx());
        vault.deposit(&env.ctx(), 42).unwrap();
        let resp = vault.get_info(&env.ctx()).unwrap();
        let info: VaultInfo = from_response(&resp).unwrap();
        assert_eq!(info.owner, ALICE);
        assert_eq!(info.balance, 42);
        assert_eq!(info.name, "vault");
    }
}
