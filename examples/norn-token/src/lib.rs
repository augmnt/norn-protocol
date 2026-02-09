//! NornToken — ERC20-style fungible token showcasing every SDK v3 feature.
//!
//! Storage: `Item` for owner/name/symbol/decimals/total_supply, `Map` for
//! balances and allowances.
//!
//! Execute: Mint, Burn, Transfer, Approve.
//! Query: Balance, Allowance, TotalSupply, Info.

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

/// Unit struct — all state lives in storage primitives.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct NornToken;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Execute {
    Mint {
        to: Address,
        amount: u128,
    },
    Burn {
        amount: u128,
    },
    Transfer {
        to: Address,
        amount: u128,
    },
    Approve {
        spender: Address,
        amount: u128,
    },
    TransferFrom {
        from: Address,
        to: Address,
        amount: u128,
    },
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Query {
    Balance { address: Address },
    Allowance { owner: Address, spender: Address },
    TotalSupply,
    Info,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct TokenInfo {
    pub owner: Address,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u128,
}

impl Contract for NornToken {
    type Exec = Execute;
    type Query = Query;

    fn init(ctx: &Context) -> Self {
        OWNER.save(&ctx.sender()).unwrap();
        TOKEN_NAME.save(&String::from("Norn Token")).unwrap();
        SYMBOL.save(&String::from("NORN")).unwrap();
        DECIMALS.save(&18u8).unwrap();
        TOTAL_SUPPLY.save(&0u128).unwrap();
        NornToken
    }

    fn execute(&mut self, ctx: &Context, msg: Execute) -> ContractResult {
        match msg {
            Execute::Mint { to, amount } => {
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
            Execute::Burn { amount } => {
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
            Execute::Transfer { to, amount } => {
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
            Execute::Approve { spender, amount } => {
                ensure_ne!(spender, ZERO_ADDRESS, "cannot approve zero address");
                let sender = ctx.sender();
                let key = allowance_key(&sender, &spender);
                ALLOWANCES.save(&key, &amount)?;

                Ok(Response::new()
                    .add_attribute("action", "approve")
                    .add_attribute("spender", addr_to_hex(&spender))
                    .add_attribute("amount", format!("{amount}")))
            }
            Execute::TransferFrom { from, to, amount } => {
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
        }
    }

    fn query(&self, _ctx: &Context, msg: Query) -> ContractResult {
        match msg {
            Query::Balance { address } => ok(BALANCES.load_or(&address, 0u128)),
            Query::Allowance { owner, spender } => {
                let key = allowance_key(&owner, &spender);
                ok(ALLOWANCES.load_or(&key, 0u128))
            }
            Query::TotalSupply => ok(TOTAL_SUPPLY.load_or(0u128)),
            Query::Info => ok(TokenInfo {
                owner: OWNER.load_or(ZERO_ADDRESS),
                name: TOKEN_NAME.load_or(String::from("")),
                symbol: SYMBOL.load_or(String::from("")),
                decimals: DECIMALS.load_or(18),
                total_supply: TOTAL_SUPPLY.load_or(0),
            }),
        }
    }
}

norn_entry!(NornToken);

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
        let token = NornToken::init(&env.ctx());
        (env, token)
    }

    #[test]
    fn test_init() {
        let (env, _token) = setup();
        assert_eq!(OWNER.load().unwrap(), ALICE);
        assert_eq!(TOTAL_SUPPLY.load().unwrap(), 0);

        let resp = _token.query(&env.ctx(), Query::Info).unwrap();
        let info: TokenInfo = from_response(&resp).unwrap();
        assert_eq!(info.symbol, "NORN");
        assert_eq!(info.decimals, 18);
    }

    #[test]
    fn test_mint() {
        let (env, mut token) = setup();
        let resp = token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: BOB,
                    amount: 1000,
                },
            )
            .unwrap();
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
        let err = token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: BOB,
                    amount: 100,
                },
            )
            .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_mint_to_zero_address() {
        let (env, mut token) = setup();
        let err = token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: ZERO_ADDRESS,
                    amount: 100,
                },
            )
            .unwrap_err();
        assert_eq!(err.message(), "cannot mint to zero address");
    }

    #[test]
    fn test_transfer() {
        let (env, mut token) = setup();
        token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: ALICE,
                    amount: 500,
                },
            )
            .unwrap();

        let resp = token
            .execute(
                &env.ctx(),
                Execute::Transfer {
                    to: BOB,
                    amount: 200,
                },
            )
            .unwrap();
        assert_attribute(&resp, "action", "transfer");
        assert_attribute(&resp, "amount", "200");

        assert_eq!(BALANCES.load_or(&ALICE, 0), 300);
        assert_eq!(BALANCES.load_or(&BOB, 0), 200);
    }

    #[test]
    fn test_transfer_insufficient() {
        let (env, mut token) = setup();
        token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: ALICE,
                    amount: 50,
                },
            )
            .unwrap();

        let err = token
            .execute(
                &env.ctx(),
                Execute::Transfer {
                    to: BOB,
                    amount: 100,
                },
            )
            .unwrap_err();
        assert!(matches!(err, ContractError::InsufficientFunds));
    }

    #[test]
    fn test_transfer_to_zero_fails() {
        let (env, mut token) = setup();
        token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: ALICE,
                    amount: 100,
                },
            )
            .unwrap();

        let err = token
            .execute(
                &env.ctx(),
                Execute::Transfer {
                    to: ZERO_ADDRESS,
                    amount: 10,
                },
            )
            .unwrap_err();
        assert_eq!(err.message(), "cannot transfer to zero address");
    }

    #[test]
    fn test_burn() {
        let (env, mut token) = setup();
        token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: ALICE,
                    amount: 300,
                },
            )
            .unwrap();

        let resp = token
            .execute(&env.ctx(), Execute::Burn { amount: 100 })
            .unwrap();
        assert_attribute(&resp, "action", "burn");

        let remaining: u128 = from_response(&resp).unwrap();
        assert_eq!(remaining, 200);
        assert_eq!(TOTAL_SUPPLY.load().unwrap(), 200);
    }

    #[test]
    fn test_burn_insufficient() {
        let (env, mut token) = setup();
        token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: ALICE,
                    amount: 10,
                },
            )
            .unwrap();

        let err = token
            .execute(&env.ctx(), Execute::Burn { amount: 50 })
            .unwrap_err();
        assert!(matches!(err, ContractError::InsufficientFunds));
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let (env, mut token) = setup();
        token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: ALICE,
                    amount: 1000,
                },
            )
            .unwrap();

        // Alice approves Bob to spend 500
        let resp = token
            .execute(
                &env.ctx(),
                Execute::Approve {
                    spender: BOB,
                    amount: 500,
                },
            )
            .unwrap();
        assert_attribute(&resp, "action", "approve");

        // Check allowance
        let resp = token
            .query(
                &env.ctx(),
                Query::Allowance {
                    owner: ALICE,
                    spender: BOB,
                },
            )
            .unwrap();
        let allowance: u128 = from_response(&resp).unwrap();
        assert_eq!(allowance, 500);

        // Bob transfers from Alice to Charlie
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
        token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: ALICE,
                    amount: 1000,
                },
            )
            .unwrap();
        token
            .execute(
                &env.ctx(),
                Execute::Approve {
                    spender: BOB,
                    amount: 100,
                },
            )
            .unwrap();

        env.set_sender(BOB);
        let err = token
            .execute(
                &env.ctx(),
                Execute::TransferFrom {
                    from: ALICE,
                    to: CHARLIE,
                    amount: 200,
                },
            )
            .unwrap_err();
        assert_eq!(err.message(), "insufficient allowance");
    }

    #[test]
    fn test_query_balance() {
        let (env, mut token) = setup();
        token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: BOB,
                    amount: 42,
                },
            )
            .unwrap();

        let resp = token
            .query(&env.ctx(), Query::Balance { address: BOB })
            .unwrap();
        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 42);

        // Non-existent balance = 0
        let resp = token
            .query(&env.ctx(), Query::Balance { address: CHARLIE })
            .unwrap();
        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 0);
    }

    #[test]
    fn test_query_total_supply() {
        let (env, mut token) = setup();
        token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: ALICE,
                    amount: 100,
                },
            )
            .unwrap();
        token
            .execute(
                &env.ctx(),
                Execute::Mint {
                    to: BOB,
                    amount: 200,
                },
            )
            .unwrap();

        let resp = token.query(&env.ctx(), Query::TotalSupply).unwrap();
        let supply: u128 = from_response(&resp).unwrap();
        assert_eq!(supply, 300);
    }
}
