//! NornToken — ERC20-style fungible token showcasing `#[norn_contract]` with
//! `Item`/`Map` storage, address helpers, and guard macros.

#![no_std]

extern crate alloc;

use norn_sdk::prelude::*;

// ── Storage layout ─────────────────────────────────────────────────────────

const OWNER: Item<Address> = Item::new("owner");
const TOKEN_NAME: Item<String> = Item::new("name");
const SYMBOL: Item<String> = Item::new("symbol");
const DECIMALS: Item<u8> = Item::new("decimals");
const TOTAL_SUPPLY: Item<u128> = Item::new("total_supply");
const BALANCES: Map<Address, u128> = Map::new("bal");
/// Allowance key = `from_address ++ to_address` (40 bytes).
const ALLOWANCES: Map<[u8; 40], u128> = Map::new("allow");

// ── Helpers ────────────────────────────────────────────────────────────────

fn allowance_key(owner: &Address, spender: &Address) -> [u8; 40] {
    let mut key = [0u8; 40];
    key[..20].copy_from_slice(owner);
    key[20..].copy_from_slice(spender);
    key
}

// ── Contract ───────────────────────────────────────────────────────────────

#[derive(BorshSerialize, BorshDeserialize)]
pub struct TokenInfo {
    pub owner: Address,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u128,
}

#[norn_contract]
pub struct NornToken;

#[norn_contract]
impl NornToken {
    #[init]
    pub fn new(ctx: &Context) -> Self {
        OWNER.save(&ctx.sender()).unwrap();
        TOKEN_NAME.save(&String::from("Norn Token")).unwrap();
        SYMBOL.save(&String::from("NORN")).unwrap();
        DECIMALS.save(&18u8).unwrap();
        TOTAL_SUPPLY.save(&0u128).unwrap();
        NornToken
    }

    #[execute]
    pub fn mint(&mut self, ctx: &Context, to: Address, amount: u128) -> ContractResult {
        let owner = OWNER.load()?;
        ctx.require_sender(&owner)?;
        ensure!(amount > 0, "mint amount must be positive");
        ensure_ne!(to, ZERO_ADDRESS, "cannot mint to zero address");

        let bal = BALANCES.load_or(&to, 0);
        let new_bal = bal.checked_add(amount).ok_or(ContractError::Overflow)?;
        BALANCES.save(&to, &new_bal)?;

        let supply = TOTAL_SUPPLY.load_or(0);
        TOTAL_SUPPLY.save(&(supply + amount))?;

        Ok(Response::new()
            .add_attribute("action", "mint")
            .add_attribute("to", addr_to_hex(&to))
            .add_attribute("amount", format!("{amount}"))
            .set_data(&new_bal))
    }

    #[execute]
    pub fn burn(&mut self, ctx: &Context, amount: u128) -> ContractResult {
        ensure!(amount > 0, "burn amount must be positive");
        let sender = ctx.sender();
        let bal = BALANCES.load_or(&sender, 0);
        ensure!(amount <= bal, ContractError::InsufficientFunds);

        BALANCES.save(&sender, &(bal - amount))?;
        let supply = TOTAL_SUPPLY.load_or(0);
        TOTAL_SUPPLY.save(&(supply - amount))?;

        Ok(Response::new()
            .add_attribute("action", "burn")
            .add_attribute("amount", format!("{amount}"))
            .set_data(&(bal - amount)))
    }

    #[execute]
    pub fn transfer(&mut self, ctx: &Context, to: Address, amount: u128) -> ContractResult {
        ensure!(amount > 0, "transfer amount must be positive");
        ensure_ne!(to, ZERO_ADDRESS, "cannot transfer to zero address");
        let sender = ctx.sender();
        ensure_ne!(sender, to, "cannot transfer to self");

        let from_bal = BALANCES.load_or(&sender, 0);
        ensure!(amount <= from_bal, ContractError::InsufficientFunds);

        let to_bal = BALANCES.load_or(&to, 0);
        BALANCES.save(&sender, &(from_bal - amount))?;
        BALANCES.save(&to, &(to_bal + amount))?;

        Ok(Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("from", addr_to_hex(&sender))
            .add_attribute("to", addr_to_hex(&to))
            .add_attribute("amount", format!("{amount}")))
    }

    #[execute]
    pub fn approve(&mut self, ctx: &Context, spender: Address, amount: u128) -> ContractResult {
        ensure_ne!(spender, ZERO_ADDRESS, "cannot approve zero address");
        let sender = ctx.sender();
        let key = allowance_key(&sender, &spender);
        ALLOWANCES.save(&key, &amount)?;

        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("spender", addr_to_hex(&spender))
            .add_attribute("amount", format!("{amount}")))
    }

    #[execute]
    pub fn transfer_from(
        &mut self,
        ctx: &Context,
        from: Address,
        to: Address,
        amount: u128,
    ) -> ContractResult {
        ensure!(amount > 0, "transfer amount must be positive");
        ensure_ne!(to, ZERO_ADDRESS, "cannot transfer to zero address");

        let spender = ctx.sender();
        let key = allowance_key(&from, &spender);
        let allowance = ALLOWANCES.load_or(&key, 0);
        ensure!(amount <= allowance, "insufficient allowance");

        let from_bal = BALANCES.load_or(&from, 0);
        ensure!(amount <= from_bal, ContractError::InsufficientFunds);

        let to_bal = BALANCES.load_or(&to, 0);
        BALANCES.save(&from, &(from_bal - amount))?;
        BALANCES.save(&to, &(to_bal + amount))?;
        ALLOWANCES.save(&key, &(allowance - amount))?;

        Ok(Response::new()
            .add_attribute("action", "transfer_from")
            .add_attribute("from", addr_to_hex(&from))
            .add_attribute("to", addr_to_hex(&to))
            .add_attribute("amount", format!("{amount}")))
    }

    #[query]
    pub fn balance(&self, _ctx: &Context, address: Address) -> ContractResult {
        ok(BALANCES.load_or(&address, 0u128))
    }

    #[query]
    pub fn allowance(&self, _ctx: &Context, owner: Address, spender: Address) -> ContractResult {
        let key = allowance_key(&owner, &spender);
        ok(ALLOWANCES.load_or(&key, 0u128))
    }

    #[query]
    pub fn total_supply(&self, _ctx: &Context) -> ContractResult {
        ok(TOTAL_SUPPLY.load_or(0u128))
    }

    #[query]
    pub fn info(&self, _ctx: &Context) -> ContractResult {
        ok(TokenInfo {
            owner: OWNER.load_or(ZERO_ADDRESS),
            name: TOKEN_NAME.load_or(String::from("")),
            symbol: SYMBOL.load_or(String::from("")),
            decimals: DECIMALS.load_or(18),
            total_supply: TOTAL_SUPPLY.load_or(0),
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
    const CHARLIE: Address = [3u8; 20];

    fn setup() -> (TestEnv, NornToken) {
        let env = TestEnv::new().with_sender(ALICE);
        let token = NornToken::new(&env.ctx());
        (env, token)
    }

    #[test]
    fn test_init() {
        let (env, token) = setup();
        assert_eq!(OWNER.load().unwrap(), ALICE);
        assert_eq!(TOTAL_SUPPLY.load().unwrap(), 0);

        let resp = token.info(&env.ctx()).unwrap();
        let info: TokenInfo = from_response(&resp).unwrap();
        assert_eq!(info.symbol, "NORN");
        assert_eq!(info.decimals, 18);
    }

    #[test]
    fn test_mint() {
        let (env, mut token) = setup();
        let resp = token.mint(&env.ctx(), BOB, 1000).unwrap();
        assert_attribute(&resp, "action", "mint");
        assert_attribute(&resp, "amount", "1000");

        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 1000);
        assert_eq!(TOTAL_SUPPLY.load().unwrap(), 1000);
    }

    #[test]
    fn test_mint_unauthorized() {
        let (env, mut token) = setup();
        env.set_sender(BOB);
        let err = token.mint(&env.ctx(), BOB, 100).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_mint_to_zero_address() {
        let (env, mut token) = setup();
        let err = token.mint(&env.ctx(), ZERO_ADDRESS, 100).unwrap_err();
        assert_eq!(err.message(), "cannot mint to zero address");
    }

    #[test]
    fn test_transfer() {
        let (env, mut token) = setup();
        token.mint(&env.ctx(), ALICE, 500).unwrap();

        let resp = token.transfer(&env.ctx(), BOB, 200).unwrap();
        assert_attribute(&resp, "action", "transfer");
        assert_attribute(&resp, "amount", "200");

        assert_eq!(BALANCES.load_or(&ALICE, 0), 300);
        assert_eq!(BALANCES.load_or(&BOB, 0), 200);
    }

    #[test]
    fn test_transfer_insufficient() {
        let (env, mut token) = setup();
        token.mint(&env.ctx(), ALICE, 50).unwrap();

        let err = token.transfer(&env.ctx(), BOB, 100).unwrap_err();
        assert!(matches!(err, ContractError::InsufficientFunds));
    }

    #[test]
    fn test_transfer_to_zero_fails() {
        let (env, mut token) = setup();
        token.mint(&env.ctx(), ALICE, 100).unwrap();

        let err = token.transfer(&env.ctx(), ZERO_ADDRESS, 10).unwrap_err();
        assert_eq!(err.message(), "cannot transfer to zero address");
    }

    #[test]
    fn test_burn() {
        let (env, mut token) = setup();
        token.mint(&env.ctx(), ALICE, 300).unwrap();

        let resp = token.burn(&env.ctx(), 100).unwrap();
        assert_attribute(&resp, "action", "burn");

        let remaining: u128 = from_response(&resp).unwrap();
        assert_eq!(remaining, 200);
        assert_eq!(TOTAL_SUPPLY.load().unwrap(), 200);
    }

    #[test]
    fn test_burn_insufficient() {
        let (env, mut token) = setup();
        token.mint(&env.ctx(), ALICE, 10).unwrap();

        let err = token.burn(&env.ctx(), 50).unwrap_err();
        assert!(matches!(err, ContractError::InsufficientFunds));
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let (env, mut token) = setup();
        token.mint(&env.ctx(), ALICE, 1000).unwrap();

        // Alice approves Bob to spend 500
        let resp = token.approve(&env.ctx(), BOB, 500).unwrap();
        assert_attribute(&resp, "action", "approve");

        // Check allowance
        let resp = token.allowance(&env.ctx(), ALICE, BOB).unwrap();
        let allow: u128 = from_response(&resp).unwrap();
        assert_eq!(allow, 500);

        // Bob transfers from Alice to Charlie
        env.set_sender(BOB);
        let resp = token.transfer_from(&env.ctx(), ALICE, CHARLIE, 200).unwrap();
        assert_attribute(&resp, "action", "transfer_from");

        assert_eq!(BALANCES.load_or(&ALICE, 0), 800);
        assert_eq!(BALANCES.load_or(&CHARLIE, 0), 200);

        // Allowance reduced
        let key = allowance_key(&ALICE, &BOB);
        assert_eq!(ALLOWANCES.load_or(&key, 0), 300);
    }

    #[test]
    fn test_transfer_from_insufficient_allowance() {
        let (env, mut token) = setup();
        token.mint(&env.ctx(), ALICE, 1000).unwrap();
        token.approve(&env.ctx(), BOB, 100).unwrap();

        env.set_sender(BOB);
        let err = token
            .transfer_from(&env.ctx(), ALICE, CHARLIE, 200)
            .unwrap_err();
        assert_eq!(err.message(), "insufficient allowance");
    }

    #[test]
    fn test_query_balance() {
        let (env, mut token) = setup();
        token.mint(&env.ctx(), BOB, 42).unwrap();

        let resp = token.balance(&env.ctx(), BOB).unwrap();
        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 42);

        // Non-existent balance = 0
        let resp = token.balance(&env.ctx(), CHARLIE).unwrap();
        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 0);
    }

    #[test]
    fn test_query_total_supply() {
        let (env, mut token) = setup();
        token.mint(&env.ctx(), ALICE, 100).unwrap();
        token.mint(&env.ctx(), BOB, 200).unwrap();

        let resp = token.total_supply(&env.ctx()).unwrap();
        let supply: u128 = from_response(&resp).unwrap();
        assert_eq!(supply, 300);
    }
}
