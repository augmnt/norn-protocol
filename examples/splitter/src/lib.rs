//! Payment Splitter — route incoming payments to multiple recipients by percentage.
//! Set once, anyone can send to it.

#![no_std]

extern crate alloc;

use alloc::{format, vec::Vec};
use norn_sdk::prelude::*;

// ── Storage ────────────────────────────────────────────────────────────

const INITIALIZED: Item<bool> = Item::new("initialized");
const CONFIG: Item<SplitterConfig> = Item::new("config");

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct Recipient {
    pub address: Address,
    pub share_bps: u64, // basis points (100 = 1%, 10000 = 100%)
}

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct SplitterConfig {
    pub name: String,
    pub creator: Address,
    pub recipients: Vec<Recipient>,
    pub created_at: u64,
}

// ── Contract ───────────────────────────────────────────────────────────

#[norn_contract]
pub struct Splitter;

#[norn_contract]
impl Splitter {
    #[init]
    pub fn new(_ctx: &Context) -> Self {
        INITIALIZED.init(&false);
        Splitter
    }

    #[execute]
    pub fn initialize(
        &mut self,
        ctx: &Context,
        name: String,
        recipients: Vec<Recipient>,
    ) -> ContractResult {
        ensure!(!INITIALIZED.load_or(false), "already initialized");
        ensure!(name.len() <= 64, "name too long (max 64)");
        ensure!(recipients.len() >= 2, "need at least 2 recipients");
        ensure!(recipients.len() <= 20, "max 20 recipients");

        let total_bps: u64 = recipients.iter().map(|r| r.share_bps).sum();
        ensure!(total_bps == 10_000, "shares must total 10000 bps (100%)");

        for r in &recipients {
            ensure!(r.share_bps > 0, "each share must be positive");
            ensure!(r.address != ZERO_ADDRESS, "recipient cannot be zero");
        }

        CONFIG.save(&SplitterConfig {
            name,
            creator: ctx.sender(),
            recipients,
            created_at: ctx.timestamp(),
        })?;
        INITIALIZED.save(&true)?;

        Ok(Response::with_action("initialize"))
    }

    #[execute]
    pub fn split(
        &mut self,
        ctx: &Context,
        token_id: TokenId,
        amount: u128,
    ) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(amount > 0, "amount must be positive");

        // Transfer full amount to contract first
        let contract = ctx.contract_address();
        ctx.transfer(&ctx.sender(), &contract, &token_id, amount);

        // Split to each recipient
        let mut distributed = 0u128;
        for (i, r) in config.recipients.iter().enumerate() {
            let share = if i == config.recipients.len() - 1 {
                // Last recipient gets remainder to avoid rounding dust
                safe_sub(amount, distributed)?
            } else {
                safe_mul(amount, r.share_bps as u128)? / 10_000
            };
            if share > 0 {
                ctx.transfer_from_contract(&r.address, &token_id, share);
                distributed = safe_add(distributed, share)?;
            }
        }

        Ok(Response::with_action("split")
            .add_attribute("amount", format!("{}", amount))
            .add_attribute("recipients", format!("{}", config.recipients.len())))
    }

    #[query]
    pub fn get_config(&self, _ctx: &Context) -> ContractResult {
        let config = CONFIG.load()?;
        ok(config)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    const TOKEN: TokenId = [42u8; 32];
    const CONTRACT_ADDR: Address = [99u8; 20];
    const CHARLIE: Address = [3u8; 20];

    fn setup() -> (TestEnv, Splitter) {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let mut s = Splitter::new(&env.ctx());
        s.initialize(
            &env.ctx(),
            "Revenue Split".into(),
            alloc::vec![
                Recipient { address: ALICE, share_bps: 6000 },
                Recipient { address: BOB, share_bps: 3000 },
                Recipient { address: CHARLIE, share_bps: 1000 },
            ],
        )
        .unwrap();
        (env, s)
    }

    #[test]
    fn test_initialize() {
        let (env, s) = setup();
        let resp = s.get_config(&env.ctx()).unwrap();
        let config: SplitterConfig = from_response(&resp).unwrap();
        assert_eq!(config.name, "Revenue Split");
        assert_eq!(config.recipients.len(), 3);
        assert_eq!(config.recipients[0].share_bps, 6000);
    }

    #[test]
    fn test_cannot_initialize_twice() {
        let (env, mut s) = setup();
        let err = s
            .initialize(
                &env.ctx(),
                "Again".into(),
                alloc::vec![
                    Recipient { address: ALICE, share_bps: 5000 },
                    Recipient { address: BOB, share_bps: 5000 },
                ],
            )
            .unwrap_err();
        assert_err_contains(&err, "already initialized");
    }

    #[test]
    fn test_shares_must_total_100() {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let mut s = Splitter::new(&env.ctx());
        let err = s
            .initialize(
                &env.ctx(),
                "Bad".into(),
                alloc::vec![
                    Recipient { address: ALICE, share_bps: 5000 },
                    Recipient { address: BOB, share_bps: 4000 },
                ],
            )
            .unwrap_err();
        assert_err_contains(&err, "shares must total 10000");
    }

    #[test]
    fn test_split() {
        let (env, mut s) = setup();
        env.set_sender(BOB);
        s.split(&env.ctx(), TOKEN, 10_000).unwrap();

        let transfers = env.transfers();
        // 1 deposit + 3 splits
        assert_eq!(transfers.len(), 4);
        // ALICE gets 60%
        assert_eq!(transfers[1].1, ALICE.to_vec());
        assert_eq!(transfers[1].3, 6000);
        // BOB gets 30%
        assert_eq!(transfers[2].1, BOB.to_vec());
        assert_eq!(transfers[2].3, 3000);
        // CHARLIE gets 10%
        assert_eq!(transfers[3].1, CHARLIE.to_vec());
        assert_eq!(transfers[3].3, 1000);
    }

    #[test]
    fn test_split_rounding() {
        // 10001 split 60/30/10 — last recipient gets remainder
        let (env, mut s) = setup();
        s.split(&env.ctx(), TOKEN, 10_001).unwrap();

        let transfers = env.transfers();
        // ALICE: 10001 * 6000 / 10000 = 6000
        assert_eq!(transfers[1].3, 6000);
        // BOB: 10001 * 3000 / 10000 = 3000
        assert_eq!(transfers[2].3, 3000);
        // CHARLIE: remainder = 10001 - 6000 - 3000 = 1001
        assert_eq!(transfers[3].3, 1001);
    }

    #[test]
    fn test_need_at_least_two_recipients() {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let mut s = Splitter::new(&env.ctx());
        let err = s
            .initialize(
                &env.ctx(),
                "Solo".into(),
                alloc::vec![Recipient { address: ALICE, share_bps: 10_000 }],
            )
            .unwrap_err();
        assert_err_contains(&err, "need at least 2 recipients");
    }
}
