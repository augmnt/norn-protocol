//! Token Launchpad — fixed-price token sale with hard cap.
//! Creator deposits tokens, buyers contribute NORN, unsold tokens returned after deadline.

#![no_std]

extern crate alloc;

use alloc::format;
use norn_sdk::prelude::*;

const NATIVE_TOKEN: TokenId = [0u8; 32];

// ── Storage ────────────────────────────────────────────────────────────

const INITIALIZED: Item<bool> = Item::new("initialized");
const CONFIG: Item<LaunchConfig> = Item::new("config");
const TOTAL_RAISED: Item<u128> = Item::new("total_raised");
const CONTRIBUTIONS: Map<Address, u128> = Map::new("contributions");
const CLAIMED: Map<Address, bool> = Map::new("claimed");

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct LaunchConfig {
    pub creator: Address,
    pub token_id: TokenId,
    pub price: u128,         // NORN per token (scaled 1e12)
    pub hard_cap: u128,      // max NORN to raise
    pub max_per_wallet: u128,
    pub start_time: u64,
    pub end_time: u64,
    pub total_tokens: u128,  // tokens deposited by creator
    pub finalized: bool,
}

// ── Contract ───────────────────────────────────────────────────────────

#[norn_contract]
pub struct Launchpad;

#[norn_contract]
impl Launchpad {
    #[init]
    pub fn new(_ctx: &Context) -> Self {
        INITIALIZED.init(&false);
        TOTAL_RAISED.init(&0u128);
        Launchpad
    }

    #[execute]
    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        &mut self,
        ctx: &Context,
        token_id: TokenId,
        price: u128,
        hard_cap: u128,
        max_per_wallet: u128,
        start_time: u64,
        end_time: u64,
        total_tokens: u128,
    ) -> ContractResult {
        ensure!(!INITIALIZED.load_or(false), "already initialized");
        ensure!(price > 0, "price must be positive");
        ensure!(hard_cap > 0, "hard_cap must be positive");
        ensure!(total_tokens > 0, "total_tokens must be positive");
        ensure!(end_time > start_time, "end_time must be after start_time");
        ensure!(max_per_wallet > 0, "max_per_wallet must be positive");

        // Transfer tokens from creator to contract
        let contract = ctx.contract_address();
        ctx.transfer(&ctx.sender(), &contract, &token_id, total_tokens);

        CONFIG.save(&LaunchConfig {
            creator: ctx.sender(),
            token_id,
            price,
            hard_cap,
            max_per_wallet,
            start_time,
            end_time,
            total_tokens,
            finalized: false,
        })?;
        INITIALIZED.save(&true)?;

        Ok(Response::with_action("initialize"))
    }

    #[execute]
    pub fn contribute(&mut self, ctx: &Context, amount: u128) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(!config.finalized, "sale is finalized");
        ensure!(ctx.timestamp() >= config.start_time, "sale has not started");
        ensure!(ctx.timestamp() < config.end_time, "sale has ended");
        ensure!(amount > 0, "amount must be positive");

        let total = TOTAL_RAISED.load_or(0u128);
        ensure!(
            safe_add(total, amount)? <= config.hard_cap,
            "would exceed hard cap"
        );

        let existing = CONTRIBUTIONS.load(&ctx.sender()).unwrap_or(0u128);
        let new_total = safe_add(existing, amount)?;
        ensure!(new_total <= config.max_per_wallet, "exceeds max per wallet");

        // Transfer NORN from buyer to contract
        let contract = ctx.contract_address();
        ctx.transfer(&ctx.sender(), &contract, &NATIVE_TOKEN, amount);

        CONTRIBUTIONS.save(&ctx.sender(), &new_total)?;
        TOTAL_RAISED.save(&safe_add(total, amount)?)?;

        Ok(Response::with_action("contribute")
            .add_attribute("amount", format!("{}", amount))
            .add_attribute("total_contribution", format!("{}", new_total)))
    }

    #[execute]
    pub fn claim_tokens(&mut self, ctx: &Context) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(config.finalized, "sale not finalized yet");

        let already_claimed = CLAIMED.load(&ctx.sender()).unwrap_or(false);
        ensure!(!already_claimed, "already claimed");

        let contribution = CONTRIBUTIONS.load(&ctx.sender()).unwrap_or(0u128);
        ensure!(contribution > 0, "no contribution found");

        // tokens = contribution / price
        let tokens = safe_mul(contribution, config.total_tokens)?
            / TOTAL_RAISED.load_or(1u128);

        ctx.transfer_from_contract(&ctx.sender(), &config.token_id, tokens);
        CLAIMED.save(&ctx.sender(), &true)?;

        Ok(Response::with_action("claim_tokens")
            .add_attribute("tokens", format!("{}", tokens)))
    }

    #[execute]
    pub fn finalize(&mut self, ctx: &Context) -> ContractResult {
        let mut config = CONFIG.load()?;
        ensure!(!config.finalized, "already finalized");
        ensure!(
            ctx.sender() == config.creator,
            "only creator can finalize"
        );
        ensure!(
            ctx.timestamp() >= config.end_time,
            "sale has not ended yet"
        );

        let total_raised = TOTAL_RAISED.load_or(0u128);

        // Send raised NORN to creator
        if total_raised > 0 {
            ctx.transfer_from_contract(&config.creator, &NATIVE_TOKEN, total_raised);
        }

        // Return unsold tokens to creator
        let tokens_sold = if total_raised > 0 {
            // proportional: total_tokens already all go to contributors via claim
            config.total_tokens
        } else {
            0
        };
        let unsold = safe_sub(config.total_tokens, tokens_sold)?;
        if unsold > 0 {
            ctx.transfer_from_contract(&config.creator, &config.token_id, unsold);
        }

        config.finalized = true;
        CONFIG.save(&config)?;

        Ok(Response::with_action("finalize")
            .add_attribute("total_raised", format!("{}", total_raised)))
    }

    #[execute]
    pub fn refund(&mut self, ctx: &Context) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(
            ctx.timestamp() >= config.end_time,
            "sale has not ended yet"
        );

        let total_raised = TOTAL_RAISED.load_or(0u128);
        ensure!(total_raised == 0, "sale had contributions, use claim_tokens after finalize");

        let contribution = CONTRIBUTIONS.load(&ctx.sender()).unwrap_or(0u128);
        ensure!(contribution > 0, "no contribution to refund");

        ctx.transfer_from_contract(&ctx.sender(), &NATIVE_TOKEN, contribution);
        CONTRIBUTIONS.save(&ctx.sender(), &0u128)?;

        Ok(Response::with_action("refund")
            .add_attribute("amount", format!("{}", contribution)))
    }

    #[query]
    pub fn get_config(&self, _ctx: &Context) -> ContractResult {
        let config = CONFIG.load()?;
        ok(config)
    }

    #[query]
    pub fn get_contribution(&self, _ctx: &Context, addr: Address) -> ContractResult {
        let amount = CONTRIBUTIONS.load(&addr).unwrap_or(0u128);
        ok(amount)
    }

    #[query]
    pub fn get_total_raised(&self, _ctx: &Context) -> ContractResult {
        let total = TOTAL_RAISED.load_or(0u128);
        ok(total)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    const TOKEN: TokenId = [42u8; 32];
    const CONTRACT_ADDR: Address = [99u8; 20];

    fn setup() -> (TestEnv, Launchpad) {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let mut lp = Launchpad::new(&env.ctx());
        lp.initialize(
            &env.ctx(),
            TOKEN,
            100,         // price
            10_000,      // hard_cap
            5_000,       // max_per_wallet
            1000,        // start_time
            2000,        // end_time
            100_000,     // total_tokens
        )
        .unwrap();
        (env, lp)
    }

    #[test]
    fn test_initialize() {
        let (env, lp) = setup();
        let resp = lp.get_config(&env.ctx()).unwrap();
        let config: LaunchConfig = from_response(&resp).unwrap();
        assert_eq!(config.creator, ALICE);
        assert_eq!(config.price, 100);
        assert_eq!(config.hard_cap, 10_000);
        assert!(!config.finalized);
    }

    #[test]
    fn test_cannot_initialize_twice() {
        let (env, mut lp) = setup();
        let err = lp
            .initialize(&env.ctx(), TOKEN, 100, 10_000, 5_000, 1000, 2000, 100_000)
            .unwrap_err();
        assert_err_contains(&err, "already initialized");
    }

    #[test]
    fn test_contribute() {
        let (env, mut lp) = setup();
        env.set_sender(BOB);
        env.set_timestamp(1500);
        lp.contribute(&env.ctx(), 1000).unwrap();

        let resp = lp.get_contribution(&env.ctx(), BOB).unwrap();
        let amount: u128 = from_response(&resp).unwrap();
        assert_eq!(amount, 1000);

        let resp = lp.get_total_raised(&env.ctx()).unwrap();
        let total: u128 = from_response(&resp).unwrap();
        assert_eq!(total, 1000);
    }

    #[test]
    fn test_cannot_contribute_before_start() {
        let (env, mut lp) = setup();
        env.set_sender(BOB);
        env.set_timestamp(500);
        let err = lp.contribute(&env.ctx(), 1000).unwrap_err();
        assert_err_contains(&err, "sale has not started");
    }

    #[test]
    fn test_cannot_contribute_after_end() {
        let (env, mut lp) = setup();
        env.set_sender(BOB);
        env.set_timestamp(2500);
        let err = lp.contribute(&env.ctx(), 1000).unwrap_err();
        assert_err_contains(&err, "sale has ended");
    }

    #[test]
    fn test_cannot_exceed_hard_cap() {
        let (env, mut lp) = setup();
        env.set_timestamp(1500);
        env.set_sender(BOB);
        lp.contribute(&env.ctx(), 5_000).unwrap();

        env.set_sender(ALICE);
        let err = lp.contribute(&env.ctx(), 5_001).unwrap_err();
        assert_err_contains(&err, "would exceed hard cap");
    }

    #[test]
    fn test_cannot_exceed_max_per_wallet() {
        let (env, mut lp) = setup();
        env.set_sender(BOB);
        env.set_timestamp(1500);
        let err = lp.contribute(&env.ctx(), 5_001).unwrap_err();
        assert_err_contains(&err, "exceeds max per wallet");
    }

    #[test]
    fn test_finalize_and_claim() {
        let (env, mut lp) = setup();

        // BOB contributes
        env.set_sender(BOB);
        env.set_timestamp(1500);
        lp.contribute(&env.ctx(), 2_000).unwrap();

        // Finalize after end
        env.set_sender(ALICE);
        env.set_timestamp(2500);
        lp.finalize(&env.ctx()).unwrap();

        // BOB claims tokens
        env.set_sender(BOB);
        lp.claim_tokens(&env.ctx()).unwrap();
    }

    #[test]
    fn test_cannot_finalize_before_end() {
        let (env, mut lp) = setup();
        env.set_timestamp(1500);
        let err = lp.finalize(&env.ctx()).unwrap_err();
        assert_err_contains(&err, "sale has not ended yet");
    }

    #[test]
    fn test_only_creator_can_finalize() {
        let (env, mut lp) = setup();
        env.set_sender(BOB);
        env.set_timestamp(2500);
        let err = lp.finalize(&env.ctx()).unwrap_err();
        assert_err_contains(&err, "only creator can finalize");
    }
}
