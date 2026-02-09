//! Norn20Token — A fungible token built entirely with the SDK standard library
//! and `#[norn_contract]` proc macro.

#![no_std]

extern crate alloc;

use norn_sdk::prelude::*;

// ── Contract ─────────────────────────────────────────────────────────────────

/// Unit struct — all state lives in the stdlib storage modules.
#[norn_contract]
pub struct Norn20Token;

#[norn_contract]
impl Norn20Token {
    #[init]
    pub fn new(
        ctx: &Context,
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: u128,
    ) -> Self {
        Ownable::init(&ctx.sender()).unwrap();
        Pausable::init().unwrap();
        Norn20::init(&name, &symbol, decimals).unwrap();
        if initial_supply > 0 {
            Norn20::mint(&ctx.sender(), initial_supply).unwrap();
        }
        Norn20Token
    }

    #[execute]
    pub fn transfer(&mut self, ctx: &Context, to: Address, amount: u128) -> ContractResult {
        Pausable::require_not_paused()?;
        Norn20::transfer(ctx, &to, amount)
    }

    #[execute]
    pub fn approve(&mut self, ctx: &Context, spender: Address, amount: u128) -> ContractResult {
        Pausable::require_not_paused()?;
        Norn20::approve(ctx, &spender, amount)
    }

    #[execute]
    pub fn transfer_from(
        &mut self,
        ctx: &Context,
        from: Address,
        to: Address,
        amount: u128,
    ) -> ContractResult {
        Pausable::require_not_paused()?;
        Norn20::transfer_from(ctx, &from, &to, amount)
    }

    #[execute]
    pub fn mint(&mut self, ctx: &Context, to: Address, amount: u128) -> ContractResult {
        Ownable::require_owner(ctx)?;
        Norn20::mint(&to, amount)
    }

    #[execute]
    pub fn burn(&mut self, ctx: &Context, from: Address, amount: u128) -> ContractResult {
        Ownable::require_owner(ctx)?;
        Norn20::burn(&from, amount)
    }

    #[execute]
    pub fn transfer_ownership(&mut self, ctx: &Context, new_owner: Address) -> ContractResult {
        Ownable::transfer_ownership(ctx, &new_owner)
    }

    #[execute]
    pub fn pause(&mut self, ctx: &Context) -> ContractResult {
        Pausable::pause(ctx)
    }

    #[execute]
    pub fn unpause(&mut self, ctx: &Context) -> ContractResult {
        Pausable::unpause(ctx)
    }

    #[query]
    pub fn balance(&self, _ctx: &Context, addr: Address) -> ContractResult {
        ok(Norn20::balance_of(&addr))
    }

    #[query]
    pub fn allowance(&self, _ctx: &Context, owner: Address, spender: Address) -> ContractResult {
        ok(Norn20::allowance(&owner, &spender))
    }

    #[query]
    pub fn total_supply(&self, _ctx: &Context) -> ContractResult {
        ok(Norn20::total_supply())
    }

    #[query]
    pub fn info(&self, _ctx: &Context) -> ContractResult {
        ok(Norn20::info()?)
    }

    #[query]
    pub fn owner(&self, _ctx: &Context) -> ContractResult {
        ok(Ownable::owner()?)
    }

    #[query]
    pub fn is_paused(&self, _ctx: &Context) -> ContractResult {
        ok(Pausable::is_paused())
    }
}

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
        let token = Norn20Token::new(
            &env.ctx(),
            String::from("Test Token"),
            String::from("TEST"),
            18,
            1_000_000,
        );
        (env, token)
    }

    #[test]
    fn test_init() {
        let (env, token) = setup();
        let resp = token.info(&env.ctx()).unwrap();
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
        let resp = token.owner(&env.ctx()).unwrap();
        let owner: Address = from_response(&resp).unwrap();
        assert_eq!(owner, ALICE);
    }

    #[test]
    fn test_transfer() {
        let (env, mut token) = setup();
        let resp = token.transfer(&env.ctx(), BOB, 1000).unwrap();
        assert_event(&resp, "Transfer");
        assert_eq!(Norn20::balance_of(&ALICE), 999_000);
        assert_eq!(Norn20::balance_of(&BOB), 1000);
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let (env, mut token) = setup();
        token.approve(&env.ctx(), BOB, 500).unwrap();
        assert_eq!(Norn20::allowance(&ALICE, &BOB), 500);

        env.set_sender(BOB);
        let resp = token.transfer_from(&env.ctx(), ALICE, CHARLIE, 200).unwrap();
        assert_event(&resp, "Transfer");
        assert_eq!(Norn20::balance_of(&CHARLIE), 200);
        assert_eq!(Norn20::allowance(&ALICE, &BOB), 300);
    }

    #[test]
    fn test_mint_owner_only() {
        let (env, mut token) = setup();
        let resp = token.mint(&env.ctx(), BOB, 5000).unwrap();
        assert_event(&resp, "Mint");
        assert_eq!(Norn20::balance_of(&BOB), 5000);

        // Non-owner can't mint
        env.set_sender(BOB);
        let err = token.mint(&env.ctx(), BOB, 100).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_burn_owner_only() {
        let (env, mut token) = setup();
        let resp = token.burn(&env.ctx(), ALICE, 500).unwrap();
        assert_event(&resp, "Burn");
        assert_eq!(Norn20::balance_of(&ALICE), 999_500);

        env.set_sender(BOB);
        let err = token.burn(&env.ctx(), ALICE, 1).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_pause_blocks_transfers() {
        let (env, mut token) = setup();
        token.pause(&env.ctx()).unwrap();

        let err = token.transfer(&env.ctx(), BOB, 10).unwrap_err();
        assert_eq!(err.message(), "contract is paused");

        // Unpause and retry
        token.unpause(&env.ctx()).unwrap();
        token.transfer(&env.ctx(), BOB, 10).unwrap();
        assert_eq!(Norn20::balance_of(&BOB), 10);
    }

    #[test]
    fn test_pause_unauthorized() {
        let (env, mut token) = setup();
        env.set_sender(BOB);
        let err = token.pause(&env.ctx()).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_transfer_ownership() {
        let (env, mut token) = setup();
        let resp = token.transfer_ownership(&env.ctx(), BOB).unwrap();
        assert_event(&resp, "OwnershipTransferred");
        assert_eq!(Ownable::owner().unwrap(), BOB);

        // Alice can no longer mint
        let err = token.mint(&env.ctx(), ALICE, 1).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));

        // Bob can now mint
        env.set_sender(BOB);
        token.mint(&env.ctx(), BOB, 100).unwrap();
    }

    #[test]
    fn test_query_is_paused() {
        let (env, mut token) = setup();
        let resp = token.is_paused(&env.ctx()).unwrap();
        let paused: bool = from_response(&resp).unwrap();
        assert!(!paused);

        token.pause(&env.ctx()).unwrap();
        let resp = token.is_paused(&env.ctx()).unwrap();
        let paused: bool = from_response(&resp).unwrap();
        assert!(paused);
    }

    #[test]
    fn test_mint_does_not_require_unpause() {
        let (env, mut token) = setup();
        token.pause(&env.ctx()).unwrap();
        // Minting is not gated by pause
        token.mint(&env.ctx(), BOB, 100).unwrap();
        assert_eq!(Norn20::balance_of(&BOB), 100);
    }
}
