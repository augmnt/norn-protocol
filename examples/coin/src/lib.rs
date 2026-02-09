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
        MINTER.save(&ctx.sender()).unwrap();
        Coin
    }

    #[execute]
    pub fn mint(&mut self, ctx: &Context, to: Address, amount: u128) -> ContractResult {
        let minter = MINTER.load()?;
        ctx.require_sender(&minter)?;
        let bal = BALANCES.load_or(&to, 0);
        BALANCES.save(&to, &(bal + amount))?;
        Ok(Response::new()
            .add_event(
                Event::new("Mint")
                    .add_attribute("to", addr_to_hex(&to))
                    .add_attribute("amount", format!("{amount}")),
            )
            .set_data(&(bal + amount)))
    }

    #[execute]
    pub fn send(&mut self, ctx: &Context, to: Address, amount: u128) -> ContractResult {
        let sender = ctx.sender();
        let from_bal = BALANCES.load_or(&sender, 0);
        ensure!(amount <= from_bal, ContractError::InsufficientFunds);
        let to_bal = BALANCES.load_or(&to, 0);
        BALANCES.save(&sender, &(from_bal - amount))?;
        BALANCES.save(&to, &(to_bal + amount))?;
        Ok(Response::new().add_event(
            Event::new("Transfer")
                .add_attribute("from", addr_to_hex(&sender))
                .add_attribute("to", addr_to_hex(&to))
                .add_attribute("amount", format!("{amount}")),
        ))
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

    const ALICE: Address = [1u8; 20];
    const BOB: Address = [2u8; 20];
    const CHARLIE: Address = [3u8; 20];

    #[test]
    fn test_init() {
        let env = TestEnv::new().with_sender(ALICE);
        let coin = Coin::new(&env.ctx());
        let resp = coin.minter(&env.ctx()).unwrap();
        let m: Address = from_response(&resp).unwrap();
        assert_eq!(m, ALICE);
    }

    #[test]
    fn test_mint() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut coin = Coin::new(&env.ctx());
        let resp = coin.mint(&env.ctx(), BOB, 1000).unwrap();
        assert_event(&resp, "Mint");
        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 1000);
    }

    #[test]
    fn test_mint_unauthorized() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut coin = Coin::new(&env.ctx());
        env.set_sender(BOB);
        let err = coin.mint(&env.ctx(), BOB, 100).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_send() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut coin = Coin::new(&env.ctx());
        coin.mint(&env.ctx(), ALICE, 500).unwrap();

        let resp = coin.send(&env.ctx(), BOB, 200).unwrap();
        assert_event(&resp, "Transfer");

        let resp = coin.balance_of(&env.ctx(), ALICE).unwrap();
        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 300);

        let resp = coin.balance_of(&env.ctx(), BOB).unwrap();
        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 200);
    }

    #[test]
    fn test_send_insufficient() {
        let env = TestEnv::new().with_sender(ALICE);
        let mut coin = Coin::new(&env.ctx());
        coin.mint(&env.ctx(), ALICE, 50).unwrap();

        let err = coin.send(&env.ctx(), BOB, 100).unwrap_err();
        assert!(matches!(err, ContractError::InsufficientFunds));
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
        assert_eq!(from_response::<u128>(&resp).unwrap(), 600);
        let resp = coin.balance_of(&env.ctx(), BOB).unwrap();
        assert_eq!(from_response::<u128>(&resp).unwrap(), 250);
        let resp = coin.balance_of(&env.ctx(), CHARLIE).unwrap();
        assert_eq!(from_response::<u128>(&resp).unwrap(), 150);
    }
}
