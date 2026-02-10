//! ERC20-equivalent fungible token standard for Norn.
//!
//! Provides `Norn20` with mint, burn, transfer, approve, and transfer_from.
//! All methods are static — state lives in storage under the `__n20:` prefix.
//!
//! ```ignore
//! use norn_sdk::prelude::*;
//!
//! fn init(ctx: &Context, msg: InitMsg) -> Self {
//!     Ownable::init(&ctx.sender()).unwrap();
//!     Norn20::init(&msg.name, &msg.symbol, msg.decimals).unwrap();
//!     if msg.initial_supply > 0 {
//!         Norn20::mint(&ctx.sender(), msg.initial_supply).unwrap();
//!     }
//!     MyToken
//! }
//! ```

use alloc::string::String;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::addr::ZERO_ADDRESS;
use crate::contract::Context;
use crate::error::ContractError;
use crate::math::safe_add;
use crate::response::{ContractResult, Event, Response};
use crate::storage::{Item, Map};
use crate::types::Address;
use crate::{ensure, ensure_ne};

// ── Storage layout ─────────────────────────────────────────────────────────

const N20_NAME: Item<String> = Item::new("__n20:name");
const N20_SYMBOL: Item<String> = Item::new("__n20:symbol");
const N20_DECIMALS: Item<u8> = Item::new("__n20:decimals");
const N20_TOTAL_SUPPLY: Item<u128> = Item::new("__n20:total_supply");
const N20_BALANCES: Map<Address, u128> = Map::new("__n20:bal");
/// Allowance key = `owner_address ++ spender_address` (40 bytes).
const N20_ALLOWANCES: Map<[u8; 40], u128> = Map::new("__n20:allow");

// ── Helpers ────────────────────────────────────────────────────────────────

fn allowance_key(owner: &Address, spender: &Address) -> [u8; 40] {
    let mut key = [0u8; 40];
    key[..20].copy_from_slice(owner);
    key[20..].copy_from_slice(spender);
    key
}

/// Token metadata returned by [`Norn20::info()`].
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Norn20Info {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u128,
}

/// ERC20-equivalent fungible token.
///
/// All methods are static — no instance needed. State is stored under
/// the `__n20:` prefix.
pub struct Norn20;

impl Norn20 {
    // ── Init ───────────────────────────────────────────────────────────

    /// Initialize token metadata. Call in your contract's `init()`.
    pub fn init(name: &str, symbol: &str, decimals: u8) -> Result<(), ContractError> {
        N20_NAME.save(&String::from(name))?;
        N20_SYMBOL.save(&String::from(symbol))?;
        N20_DECIMALS.save(&decimals)?;
        N20_TOTAL_SUPPLY.save(&0u128)?;
        Ok(())
    }

    // ── Queries ────────────────────────────────────────────────────────

    /// Get the token name.
    pub fn name() -> Result<String, ContractError> {
        N20_NAME.load()
    }

    /// Get the token symbol.
    pub fn symbol() -> Result<String, ContractError> {
        N20_SYMBOL.load()
    }

    /// Get the number of decimals.
    pub fn decimals() -> Result<u8, ContractError> {
        N20_DECIMALS.load()
    }

    /// Get the total supply.
    pub fn total_supply() -> u128 {
        N20_TOTAL_SUPPLY.load_or(0)
    }

    /// Get the balance of an address.
    pub fn balance_of(addr: &Address) -> u128 {
        N20_BALANCES.load_or(addr, 0)
    }

    /// Get the allowance granted by `owner` to `spender`.
    pub fn allowance(owner: &Address, spender: &Address) -> u128 {
        let key = allowance_key(owner, spender);
        N20_ALLOWANCES.load_or(&key, 0)
    }

    /// Get full token metadata.
    pub fn info() -> Result<Norn20Info, ContractError> {
        Ok(Norn20Info {
            name: N20_NAME.load_or(String::new()),
            symbol: N20_SYMBOL.load_or(String::new()),
            decimals: N20_DECIMALS.load_or(18),
            total_supply: N20_TOTAL_SUPPLY.load_or(0),
        })
    }

    // ── Mutations ──────────────────────────────────────────────────────

    /// Mint tokens to an address. Returns a `Response` with a `Mint` event.
    ///
    /// **Note**: Does not check authorization — the caller should enforce
    /// who is allowed to mint (e.g., `Ownable::require_owner(ctx)?`).
    pub fn mint(to: &Address, amount: u128) -> ContractResult {
        ensure!(amount > 0, "mint amount must be positive");
        ensure_ne!(*to, ZERO_ADDRESS, "cannot mint to zero address");

        let bal = N20_BALANCES.load_or(to, 0);
        let new_bal = safe_add(bal, amount)?;
        N20_BALANCES.save(to, &new_bal)?;

        let supply = N20_TOTAL_SUPPLY.load_or(0);
        N20_TOTAL_SUPPLY.save(&(supply + amount))?;

        Ok(Response::new()
            .add_event(
                Event::new("Mint")
                    .add_address("to", to)
                    .add_u128("amount", amount),
            )
            .set_data(&new_bal))
    }

    /// Burn tokens from an address. Returns a `Response` with a `Burn` event.
    ///
    /// **Note**: Does not check authorization — the caller should verify
    /// that the sender owns the tokens being burned.
    pub fn burn(from: &Address, amount: u128) -> ContractResult {
        ensure!(amount > 0, "burn amount must be positive");

        let bal = N20_BALANCES.load_or(from, 0);
        ensure!(amount <= bal, ContractError::InsufficientFunds);

        N20_BALANCES.save(from, &(bal - amount))?;
        let supply = N20_TOTAL_SUPPLY.load_or(0);
        N20_TOTAL_SUPPLY.save(&(supply - amount))?;

        Ok(Response::new()
            .add_event(
                Event::new("Burn")
                    .add_address("from", from)
                    .add_u128("amount", amount),
            )
            .set_data(&(bal - amount)))
    }

    /// Transfer tokens from sender to `to`. Returns a `Response` with a `Transfer` event.
    pub fn transfer(ctx: &Context, to: &Address, amount: u128) -> ContractResult {
        ensure!(amount > 0, "transfer amount must be positive");
        ensure_ne!(*to, ZERO_ADDRESS, "cannot transfer to zero address");

        let sender = ctx.sender();
        ensure_ne!(sender, *to, "cannot transfer to self");

        let from_bal = N20_BALANCES.load_or(&sender, 0);
        ensure!(amount <= from_bal, ContractError::InsufficientFunds);

        let to_bal = N20_BALANCES.load_or(to, 0);
        N20_BALANCES.save(&sender, &(from_bal - amount))?;
        N20_BALANCES.save(to, &(to_bal + amount))?;

        Ok(Response::new().add_event(
            Event::new("Transfer")
                .add_address("from", &sender)
                .add_address("to", to)
                .add_u128("amount", amount),
        ))
    }

    /// Approve `spender` to spend `amount` on behalf of the sender.
    pub fn approve(ctx: &Context, spender: &Address, amount: u128) -> ContractResult {
        ensure_ne!(*spender, ZERO_ADDRESS, "cannot approve zero address");
        let sender = ctx.sender();
        let key = allowance_key(&sender, spender);
        N20_ALLOWANCES.save(&key, &amount)?;

        Ok(Response::new().add_event(
            Event::new("Approval")
                .add_address("owner", &sender)
                .add_address("spender", spender)
                .add_u128("amount", amount),
        ))
    }

    /// Transfer tokens from `from` to `to` using the caller's allowance.
    pub fn transfer_from(
        ctx: &Context,
        from: &Address,
        to: &Address,
        amount: u128,
    ) -> ContractResult {
        ensure!(amount > 0, "transfer amount must be positive");
        ensure_ne!(*to, ZERO_ADDRESS, "cannot transfer to zero address");

        let spender = ctx.sender();
        let key = allowance_key(from, &spender);
        let allowance = N20_ALLOWANCES.load_or(&key, 0);
        ensure!(amount <= allowance, "insufficient allowance");

        let from_bal = N20_BALANCES.load_or(from, 0);
        ensure!(amount <= from_bal, ContractError::InsufficientFunds);

        let to_bal = N20_BALANCES.load_or(to, 0);
        N20_BALANCES.save(from, &(from_bal - amount))?;
        N20_BALANCES.save(to, &(to_bal + amount))?;
        N20_ALLOWANCES.save(&key, &(allowance - amount))?;

        Ok(Response::new().add_event(
            Event::new("Transfer")
                .add_address("from", from)
                .add_address("to", to)
                .add_u128("amount", amount),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    fn setup() -> TestEnv {
        let env = TestEnv::new().with_sender(ALICE);
        Norn20::init("Test Token", "TEST", 18).unwrap();
        env
    }

    #[test]
    fn test_init() {
        let _env = setup();
        assert_eq!(Norn20::name().unwrap(), "Test Token");
        assert_eq!(Norn20::symbol().unwrap(), "TEST");
        assert_eq!(Norn20::decimals().unwrap(), 18);
        assert_eq!(Norn20::total_supply(), 0);
    }

    #[test]
    fn test_info() {
        let _env = setup();
        let info = Norn20::info().unwrap();
        assert_eq!(info.name, "Test Token");
        assert_eq!(info.symbol, "TEST");
        assert_eq!(info.decimals, 18);
        assert_eq!(info.total_supply, 0);
    }

    #[test]
    fn test_mint() {
        let _env = setup();
        let resp = Norn20::mint(&ALICE, 1000).unwrap();
        assert_event(&resp, "Mint");
        assert_eq!(Norn20::balance_of(&ALICE), 1000);
        assert_eq!(Norn20::total_supply(), 1000);
    }

    #[test]
    fn test_mint_zero_fails() {
        let _env = setup();
        let err = Norn20::mint(&ALICE, 0).unwrap_err();
        assert_eq!(err.message(), "mint amount must be positive");
    }

    #[test]
    fn test_mint_to_zero_fails() {
        let _env = setup();
        let err = Norn20::mint(&ZERO_ADDRESS, 100).unwrap_err();
        assert_eq!(err.message(), "cannot mint to zero address");
    }

    #[test]
    fn test_burn() {
        let _env = setup();
        Norn20::mint(&ALICE, 500).unwrap();
        let resp = Norn20::burn(&ALICE, 200).unwrap();
        assert_event(&resp, "Burn");
        assert_eq!(Norn20::balance_of(&ALICE), 300);
        assert_eq!(Norn20::total_supply(), 300);
    }

    #[test]
    fn test_burn_insufficient() {
        let _env = setup();
        Norn20::mint(&ALICE, 100).unwrap();
        let err = Norn20::burn(&ALICE, 200).unwrap_err();
        assert_eq!(err, ContractError::InsufficientFunds);
    }

    #[test]
    fn test_transfer() {
        let env = setup();
        Norn20::mint(&ALICE, 1000).unwrap();
        let resp = Norn20::transfer(&env.ctx(), &BOB, 300).unwrap();
        assert_event(&resp, "Transfer");
        assert_eq!(Norn20::balance_of(&ALICE), 700);
        assert_eq!(Norn20::balance_of(&BOB), 300);
    }

    #[test]
    fn test_transfer_insufficient() {
        let env = setup();
        Norn20::mint(&ALICE, 50).unwrap();
        let err = Norn20::transfer(&env.ctx(), &BOB, 100).unwrap_err();
        assert_eq!(err, ContractError::InsufficientFunds);
    }

    #[test]
    fn test_transfer_to_zero_fails() {
        let env = setup();
        Norn20::mint(&ALICE, 100).unwrap();
        let err = Norn20::transfer(&env.ctx(), &ZERO_ADDRESS, 50).unwrap_err();
        assert_eq!(err.message(), "cannot transfer to zero address");
    }

    #[test]
    fn test_transfer_to_self_fails() {
        let env = setup();
        Norn20::mint(&ALICE, 100).unwrap();
        let err = Norn20::transfer(&env.ctx(), &ALICE, 50).unwrap_err();
        assert_eq!(err.message(), "cannot transfer to self");
    }

    #[test]
    fn test_approve_and_allowance() {
        let env = setup();
        let resp = Norn20::approve(&env.ctx(), &BOB, 500).unwrap();
        assert_event(&resp, "Approval");
        assert_eq!(Norn20::allowance(&ALICE, &BOB), 500);
    }

    #[test]
    fn test_transfer_from() {
        let env = setup();
        Norn20::mint(&ALICE, 1000).unwrap();
        Norn20::approve(&env.ctx(), &BOB, 500).unwrap();

        env.set_sender(BOB);
        let resp = Norn20::transfer_from(&env.ctx(), &ALICE, &CHARLIE, 200).unwrap();
        assert_event(&resp, "Transfer");
        assert_eq!(Norn20::balance_of(&ALICE), 800);
        assert_eq!(Norn20::balance_of(&CHARLIE), 200);
        assert_eq!(Norn20::allowance(&ALICE, &BOB), 300);
    }

    #[test]
    fn test_transfer_from_insufficient_allowance() {
        let env = setup();
        Norn20::mint(&ALICE, 1000).unwrap();
        Norn20::approve(&env.ctx(), &BOB, 100).unwrap();

        env.set_sender(BOB);
        let err = Norn20::transfer_from(&env.ctx(), &ALICE, &CHARLIE, 200).unwrap_err();
        assert_eq!(err.message(), "insufficient allowance");
    }

    #[test]
    fn test_transfer_from_insufficient_balance() {
        let env = setup();
        Norn20::mint(&ALICE, 100).unwrap();
        Norn20::approve(&env.ctx(), &BOB, 500).unwrap();

        env.set_sender(BOB);
        let err = Norn20::transfer_from(&env.ctx(), &ALICE, &CHARLIE, 200).unwrap_err();
        assert_eq!(err, ContractError::InsufficientFunds);
    }

    #[test]
    fn test_balance_of_nonexistent() {
        let _env = setup();
        assert_eq!(Norn20::balance_of(&BOB), 0);
    }

    #[test]
    fn test_multiple_mints() {
        let _env = setup();
        Norn20::mint(&ALICE, 100).unwrap();
        Norn20::mint(&BOB, 200).unwrap();
        Norn20::mint(&ALICE, 50).unwrap();
        assert_eq!(Norn20::balance_of(&ALICE), 150);
        assert_eq!(Norn20::balance_of(&BOB), 200);
        assert_eq!(Norn20::total_supply(), 350);
    }

    #[test]
    fn test_approve_zero_address_fails() {
        let env = setup();
        let err = Norn20::approve(&env.ctx(), &ZERO_ADDRESS, 100).unwrap_err();
        assert_eq!(err.message(), "cannot approve zero address");
    }
}
