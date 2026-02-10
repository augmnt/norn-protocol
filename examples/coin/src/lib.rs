//! Coin — a minimal token contract mirroring Solidity's intro Coin example.
//!
//! Demonstrates that `#[norn_contract]` achieves structural parity with
//! Solidity: just struct fields and annotated methods.

#![no_std]

extern crate alloc;

use norn_sdk::prelude::*;

const MINTER: Item<Address> = Item::new("minter");
const BALANCES: Map<Address, u128> = Map::new("bal");

#[norn_contract]
pub struct Coin;

#[norn_contract]
impl Coin {
    #[init]
    pub fn new(ctx: &Context) -> Self {
        MINTER.init(&ctx.sender());
        Coin
    }

    #[execute]
    pub fn mint(&mut self, ctx: &Context, to: Address, amount: u128) -> ContractResult {
        let minter = MINTER.load()?;
        ctx.require_sender(&minter)?;
        let bal = BALANCES.load_or(&to, 0);
        let new_bal = safe_add(bal, amount)?;
        BALANCES.save(&to, &new_bal)?;
        Ok(Response::new()
            .add_event(event!("Mint", to: to, amount: amount))
            .set_data(&new_bal))
    }

    #[execute]
    pub fn send(&mut self, ctx: &Context, to: Address, amount: u128) -> ContractResult {
        let sender = ctx.sender();
        let from_bal = BALANCES.load_or(&sender, 0);
        let new_from = safe_sub(from_bal, amount)?;
        let to_bal = BALANCES.load_or(&to, 0);
        BALANCES.save(&sender, &new_from)?;
        BALANCES.save(&to, &(to_bal + amount))?;
        Ok(Response::new()
            .add_event(event!("Transfer", from: sender, to: to, amount: amount)))
    }

    #[query]
    pub fn balance_of(&self, _ctx: &Context, addr: Address) -> ContractResult {
        ok(BALANCES.load_or(&addr, 0u128))
    }

    #[query]
    pub fn minter(&self, _ctx: &Context) -> ContractResult {
        ok(MINTER.load_or(ZERO_ADDRESS))
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    #[test]
    fn test_init() {
        let env = TestEnv::new().with_sender(ALICE);
        let coin = Coin::new(&env.ctx());
        let resp = coin.minter(&env.ctx()).unwrap();
        assert_data::<Address>(&resp, &ALICE);
    }

    #[test]
    fn test_mint() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut coin = Coin::new(&env.ctx());
        let resp = coin.mint(&env.ctx(), BOB, 1000).unwrap();
        assert_event(&resp, "Mint");
        assert_data::<u128>(&resp, &1000);
    }

    #[test]
    fn test_mint_unauthorized() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut coin = Coin::new(&env.ctx());
        env.set_sender(BOB);
        let err = coin.mint(&env.ctx(), BOB, 100).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized);
    }

    #[test]
    fn test_send() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut coin = Coin::new(&env.ctx());
        coin.mint(&env.ctx(), ALICE, 500).unwrap();

        let resp = coin.send(&env.ctx(), BOB, 200).unwrap();
        assert_event(&resp, "Transfer");

        let resp = coin.balance_of(&env.ctx(), ALICE).unwrap();
        assert_data::<u128>(&resp, &300);

        let resp = coin.balance_of(&env.ctx(), BOB).unwrap();
        assert_data::<u128>(&resp, &200);
    }

    #[test]
    fn test_send_insufficient() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut coin = Coin::new(&env.ctx());
        coin.mint(&env.ctx(), ALICE, 50).unwrap();

        let err = coin.send(&env.ctx(), BOB, 100).unwrap_err();
        assert_eq!(err, ContractError::InsufficientFunds);
    }

    #[test]
    fn test_multi_hop() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut coin = Coin::new(&env.ctx());
        coin.mint(&env.ctx(), ALICE, 1000).unwrap();

        coin.send(&env.ctx(), BOB, 400).unwrap();
        env.set_sender(BOB);
        coin.send(&env.ctx(), CHARLIE, 150).unwrap();

        let resp = coin.balance_of(&env.ctx(), ALICE).unwrap();
        assert_data::<u128>(&resp, &600);
        let resp = coin.balance_of(&env.ctx(), BOB).unwrap();
        assert_data::<u128>(&resp, &250);
        let resp = coin.balance_of(&env.ctx(), CHARLIE).unwrap();
        assert_data::<u128>(&resp, &150);
    }
}
