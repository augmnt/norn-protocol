//! Airdrop Distributor — upload addresses + amounts, recipients claim allocations.

#![no_std]

extern crate alloc;

use alloc::{format, vec::Vec};
use norn_sdk::prelude::*;

// ── Storage ────────────────────────────────────────────────────────────

const INITIALIZED: Item<bool> = Item::new("initialized");
const CONFIG: Item<AirdropConfig> = Item::new("config");
const ALLOCATIONS: Map<Address, u128> = Map::new("allocations");
const CLAIMED: Map<Address, bool> = Map::new("claimed");

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct AirdropConfig {
    pub creator: Address,
    pub token_id: TokenId,
    pub total_amount: u128,
    pub claimed_amount: u128,
    pub recipient_count: u64,
    pub finalized: bool,
    pub created_at: u64,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct Allocation {
    pub address: Address,
    pub amount: u128,
}

// ── Contract ───────────────────────────────────────────────────────────

#[norn_contract]
pub struct Airdrop;

#[norn_contract]
impl Airdrop {
    #[init]
    pub fn new(_ctx: &Context) -> Self {
        INITIALIZED.init(&false);
        Airdrop
    }

    #[execute]
    pub fn initialize(
        &mut self,
        ctx: &Context,
        token_id: TokenId,
        total_amount: u128,
    ) -> ContractResult {
        ensure!(!INITIALIZED.load_or(false), "already initialized");
        ensure!(total_amount > 0, "total_amount must be positive");

        // Transfer tokens to contract
        let contract = ctx.contract_address();
        ctx.transfer(&ctx.sender(), &contract, &token_id, total_amount);

        CONFIG.save(&AirdropConfig {
            creator: ctx.sender(),
            token_id,
            total_amount,
            claimed_amount: 0,
            recipient_count: 0,
            finalized: false,
            created_at: ctx.timestamp(),
        })?;
        INITIALIZED.save(&true)?;

        Ok(Response::with_action("initialize"))
    }

    #[execute]
    pub fn add_recipients(
        &mut self,
        ctx: &Context,
        recipients: Vec<Allocation>,
    ) -> ContractResult {
        let mut config = CONFIG.load()?;
        ensure!(!config.finalized, "airdrop is finalized");
        ensure!(
            ctx.sender() == config.creator,
            "only creator can add recipients"
        );
        ensure!(!recipients.is_empty(), "recipients list is empty");
        ensure!(recipients.len() <= 100, "max 100 recipients per batch");

        for alloc in &recipients {
            ensure!(alloc.amount > 0, "allocation must be positive");
            ensure!(alloc.address != ZERO_ADDRESS, "cannot allocate to zero");

            let existing = ALLOCATIONS.load(&alloc.address).unwrap_or(0u128);
            if existing == 0 {
                config.recipient_count = safe_add_u64(config.recipient_count, 1)?;
            }
            ALLOCATIONS.save(&alloc.address, &safe_add(existing, alloc.amount)?)?;
        }

        CONFIG.save(&config)?;

        Ok(Response::with_action("add_recipients")
            .add_attribute("count", format!("{}", recipients.len())))
    }

    #[execute]
    pub fn finalize(&mut self, ctx: &Context) -> ContractResult {
        let mut config = CONFIG.load()?;
        ensure!(!config.finalized, "already finalized");
        ensure!(ctx.sender() == config.creator, "only creator can finalize");

        config.finalized = true;
        CONFIG.save(&config)?;

        Ok(Response::with_action("finalize"))
    }

    #[execute]
    pub fn claim(&mut self, ctx: &Context) -> ContractResult {
        let mut config = CONFIG.load()?;
        ensure!(config.finalized, "airdrop not finalized yet");

        let already_claimed = CLAIMED.load(&ctx.sender()).unwrap_or(false);
        ensure!(!already_claimed, "already claimed");

        let allocation = ALLOCATIONS.load(&ctx.sender()).unwrap_or(0u128);
        ensure!(allocation > 0, "no allocation found");

        ctx.transfer_from_contract(&ctx.sender(), &config.token_id, allocation);
        CLAIMED.save(&ctx.sender(), &true)?;
        config.claimed_amount = safe_add(config.claimed_amount, allocation)?;
        CONFIG.save(&config)?;

        Ok(Response::with_action("claim")
            .add_attribute("amount", format!("{}", allocation)))
    }

    #[execute]
    pub fn reclaim_remaining(&mut self, ctx: &Context) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(config.finalized, "airdrop not finalized yet");
        ensure!(ctx.sender() == config.creator, "only creator can reclaim");

        let remaining = safe_sub(config.total_amount, config.claimed_amount)?;
        ensure!(remaining > 0, "nothing to reclaim");

        ctx.transfer_from_contract(&config.creator, &config.token_id, remaining);

        Ok(Response::with_action("reclaim_remaining")
            .add_attribute("amount", format!("{}", remaining)))
    }

    #[query]
    pub fn get_config(&self, _ctx: &Context) -> ContractResult {
        let config = CONFIG.load()?;
        ok(config)
    }

    #[query]
    pub fn get_allocation(&self, _ctx: &Context, addr: Address) -> ContractResult {
        let amount = ALLOCATIONS.load(&addr).unwrap_or(0u128);
        ok(amount)
    }

    #[query]
    pub fn is_claimed(&self, _ctx: &Context, addr: Address) -> ContractResult {
        let claimed = CLAIMED.load(&addr).unwrap_or(false);
        ok(claimed)
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

    fn setup() -> (TestEnv, Airdrop) {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let mut ad = Airdrop::new(&env.ctx());
        ad.initialize(&env.ctx(), TOKEN, 100_000).unwrap();
        (env, ad)
    }

    #[test]
    fn test_initialize() {
        let (env, ad) = setup();
        let resp = ad.get_config(&env.ctx()).unwrap();
        let config: AirdropConfig = from_response(&resp).unwrap();
        assert_eq!(config.total_amount, 100_000);
        assert_eq!(config.recipient_count, 0);
        assert!(!config.finalized);
    }

    #[test]
    fn test_add_recipients() {
        let (env, mut ad) = setup();
        ad.add_recipients(
            &env.ctx(),
            alloc::vec![
                Allocation { address: BOB, amount: 5_000 },
                Allocation { address: CHARLIE, amount: 3_000 },
            ],
        )
        .unwrap();

        let resp = ad.get_allocation(&env.ctx(), BOB).unwrap();
        let amount: u128 = from_response(&resp).unwrap();
        assert_eq!(amount, 5_000);

        let resp = ad.get_config(&env.ctx()).unwrap();
        let config: AirdropConfig = from_response(&resp).unwrap();
        assert_eq!(config.recipient_count, 2);
    }

    #[test]
    fn test_cannot_add_if_not_creator() {
        let (env, mut ad) = setup();
        env.set_sender(BOB);
        let err = ad
            .add_recipients(
                &env.ctx(),
                alloc::vec![Allocation { address: CHARLIE, amount: 1000 }],
            )
            .unwrap_err();
        assert_err_contains(&err, "only creator can add recipients");
    }

    #[test]
    fn test_claim() {
        let (env, mut ad) = setup();
        ad.add_recipients(
            &env.ctx(),
            alloc::vec![Allocation { address: BOB, amount: 5_000 }],
        )
        .unwrap();
        ad.finalize(&env.ctx()).unwrap();

        env.set_sender(BOB);
        ad.claim(&env.ctx()).unwrap();

        let resp = ad.is_claimed(&env.ctx(), BOB).unwrap();
        let claimed: bool = from_response(&resp).unwrap();
        assert!(claimed);
    }

    #[test]
    fn test_cannot_claim_twice() {
        let (env, mut ad) = setup();
        ad.add_recipients(
            &env.ctx(),
            alloc::vec![Allocation { address: BOB, amount: 5_000 }],
        )
        .unwrap();
        ad.finalize(&env.ctx()).unwrap();

        env.set_sender(BOB);
        ad.claim(&env.ctx()).unwrap();
        let err = ad.claim(&env.ctx()).unwrap_err();
        assert_err_contains(&err, "already claimed");
    }

    #[test]
    fn test_cannot_claim_before_finalize() {
        let (env, mut ad) = setup();
        ad.add_recipients(
            &env.ctx(),
            alloc::vec![Allocation { address: BOB, amount: 5_000 }],
        )
        .unwrap();

        env.set_sender(BOB);
        let err = ad.claim(&env.ctx()).unwrap_err();
        assert_err_contains(&err, "airdrop not finalized yet");
    }

    #[test]
    fn test_no_allocation() {
        let (env, mut ad) = setup();
        ad.finalize(&env.ctx()).unwrap();

        env.set_sender(BOB);
        let err = ad.claim(&env.ctx()).unwrap_err();
        assert_err_contains(&err, "no allocation found");
    }

    #[test]
    fn test_reclaim_remaining() {
        let (env, mut ad) = setup();
        ad.add_recipients(
            &env.ctx(),
            alloc::vec![Allocation { address: BOB, amount: 5_000 }],
        )
        .unwrap();
        ad.finalize(&env.ctx()).unwrap();

        // BOB claims 5000, 95000 remains
        env.set_sender(BOB);
        ad.claim(&env.ctx()).unwrap();

        env.set_sender(ALICE);
        ad.reclaim_remaining(&env.ctx()).unwrap();
    }
}
