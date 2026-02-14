//! Crowdfund — all-or-nothing fundraising with a goal and deadline.
//! If goal met, creator gets funds. If not, contributors get refunds.

#![no_std]

extern crate alloc;

use alloc::format;
use norn_sdk::prelude::*;

// ── Storage ────────────────────────────────────────────────────────────

const INITIALIZED: Item<bool> = Item::new("initialized");
const CONFIG: Item<CrowdfundConfig> = Item::new("config");
const TOTAL_RAISED: Item<u128> = Item::new("total_raised");
const CONTRIBUTIONS: Map<Address, u128> = Map::new("contributions");
const CONTRIBUTOR_COUNT: Item<u64> = Item::new("contributor_count");

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone, PartialEq)]
pub enum CampaignStatus {
    Active,
    Succeeded,
    Failed,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct CrowdfundConfig {
    pub creator: Address,
    pub title: String,
    pub description: String,
    pub token_id: TokenId,
    pub goal: u128,
    pub deadline: u64,
    pub status: CampaignStatus,
    pub created_at: u64,
}

// ── Contract ───────────────────────────────────────────────────────────

#[norn_contract]
pub struct Crowdfund;

#[norn_contract]
impl Crowdfund {
    #[init]
    pub fn new(_ctx: &Context) -> Self {
        INITIALIZED.init(&false);
        TOTAL_RAISED.init(&0u128);
        CONTRIBUTOR_COUNT.init(&0u64);
        Crowdfund
    }

    #[execute]
    pub fn initialize(
        &mut self,
        ctx: &Context,
        title: String,
        description: String,
        token_id: TokenId,
        goal: u128,
        deadline: u64,
    ) -> ContractResult {
        ensure!(!INITIALIZED.load_or(false), "already initialized");
        ensure!(title.len() <= 128, "title too long (max 128)");
        ensure!(description.len() <= 512, "description too long (max 512)");
        ensure!(goal > 0, "goal must be positive");
        ensure!(deadline > ctx.timestamp(), "deadline must be in the future");

        CONFIG.save(&CrowdfundConfig {
            creator: ctx.sender(),
            title,
            description,
            token_id,
            goal,
            deadline,
            status: CampaignStatus::Active,
            created_at: ctx.timestamp(),
        })?;
        INITIALIZED.save(&true)?;

        Ok(Response::with_action("initialize"))
    }

    #[execute]
    pub fn contribute(&mut self, ctx: &Context, amount: u128) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(config.status == CampaignStatus::Active, "campaign is not active");
        ensure!(ctx.timestamp() < config.deadline, "campaign has ended");
        ensure!(amount > 0, "amount must be positive");

        let contract = ctx.contract_address();
        ctx.transfer(&ctx.sender(), &contract, &config.token_id, amount);

        let existing = CONTRIBUTIONS.load(&ctx.sender()).unwrap_or(0u128);
        if existing == 0 {
            let count = CONTRIBUTOR_COUNT.load_or(0u64);
            CONTRIBUTOR_COUNT.save(&safe_add_u64(count, 1)?)?;
        }
        CONTRIBUTIONS.save(&ctx.sender(), &safe_add(existing, amount)?)?;
        let total = TOTAL_RAISED.load_or(0u128);
        TOTAL_RAISED.save(&safe_add(total, amount)?)?;

        Ok(Response::with_action("contribute")
            .add_attribute("amount", format!("{}", amount)))
    }

    #[execute]
    pub fn finalize(&mut self, ctx: &Context) -> ContractResult {
        let mut config = CONFIG.load()?;
        ensure!(config.status == CampaignStatus::Active, "already finalized");
        ensure!(
            ctx.timestamp() >= config.deadline,
            "campaign has not ended yet"
        );

        let total = TOTAL_RAISED.load_or(0u128);

        if total >= config.goal {
            // Success — send funds to creator
            ctx.transfer_from_contract(&config.creator, &config.token_id, total);
            config.status = CampaignStatus::Succeeded;
        } else {
            config.status = CampaignStatus::Failed;
        }

        CONFIG.save(&config)?;

        Ok(Response::with_action("finalize")
            .add_attribute("status", format!("{:?}", config.status))
            .add_attribute("total_raised", format!("{}", total)))
    }

    #[execute]
    pub fn refund(&mut self, ctx: &Context) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(
            config.status == CampaignStatus::Failed,
            "refunds only available for failed campaigns"
        );

        let contribution = CONTRIBUTIONS.load(&ctx.sender()).unwrap_or(0u128);
        ensure!(contribution > 0, "no contribution to refund");

        ctx.transfer_from_contract(&ctx.sender(), &config.token_id, contribution);
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

    #[query]
    pub fn get_contributor_count(&self, _ctx: &Context) -> ContractResult {
        let count = CONTRIBUTOR_COUNT.load_or(0u64);
        ok(count)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    const TOKEN: TokenId = [42u8; 32];
    const CONTRACT_ADDR: Address = [99u8; 20];

    fn setup() -> (TestEnv, Crowdfund) {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let mut cf = Crowdfund::new(&env.ctx());
        cf.initialize(
            &env.ctx(),
            "Build a Bridge".into(),
            "Community bridge project".into(),
            TOKEN,
            10_000,
            2000,
        )
        .unwrap();
        (env, cf)
    }

    #[test]
    fn test_initialize() {
        let (env, cf) = setup();
        let resp = cf.get_config(&env.ctx()).unwrap();
        let config: CrowdfundConfig = from_response(&resp).unwrap();
        assert_eq!(config.title, "Build a Bridge");
        assert_eq!(config.goal, 10_000);
        assert_eq!(config.status, CampaignStatus::Active);
    }

    #[test]
    fn test_contribute() {
        let (env, mut cf) = setup();
        env.set_sender(BOB);
        env.set_timestamp(1500);
        cf.contribute(&env.ctx(), 5_000).unwrap();

        let resp = cf.get_contribution(&env.ctx(), BOB).unwrap();
        let amount: u128 = from_response(&resp).unwrap();
        assert_eq!(amount, 5_000);

        let resp = cf.get_contributor_count(&env.ctx()).unwrap();
        let count: u64 = from_response(&resp).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_cannot_contribute_after_deadline() {
        let (env, mut cf) = setup();
        env.set_sender(BOB);
        env.set_timestamp(3000);
        let err = cf.contribute(&env.ctx(), 1000).unwrap_err();
        assert_err_contains(&err, "campaign has ended");
    }

    #[test]
    fn test_finalize_success() {
        let (env, mut cf) = setup();
        env.set_sender(BOB);
        env.set_timestamp(1500);
        cf.contribute(&env.ctx(), 10_000).unwrap();

        env.set_sender(ALICE);
        env.set_timestamp(2500);
        cf.finalize(&env.ctx()).unwrap();

        let resp = cf.get_config(&env.ctx()).unwrap();
        let config: CrowdfundConfig = from_response(&resp).unwrap();
        assert_eq!(config.status, CampaignStatus::Succeeded);

        // Creator got the funds
        let transfers = env.transfers();
        assert_eq!(transfers.last().unwrap().1, ALICE.to_vec());
        assert_eq!(transfers.last().unwrap().3, 10_000);
    }

    #[test]
    fn test_finalize_failed_and_refund() {
        let (env, mut cf) = setup();
        env.set_sender(BOB);
        env.set_timestamp(1500);
        cf.contribute(&env.ctx(), 5_000).unwrap(); // below goal

        env.set_sender(ALICE);
        env.set_timestamp(2500);
        cf.finalize(&env.ctx()).unwrap();

        let resp = cf.get_config(&env.ctx()).unwrap();
        let config: CrowdfundConfig = from_response(&resp).unwrap();
        assert_eq!(config.status, CampaignStatus::Failed);

        // BOB can refund
        env.set_sender(BOB);
        cf.refund(&env.ctx()).unwrap();

        let resp = cf.get_contribution(&env.ctx(), BOB).unwrap();
        let amount: u128 = from_response(&resp).unwrap();
        assert_eq!(amount, 0);
    }

    #[test]
    fn test_cannot_refund_if_succeeded() {
        let (env, mut cf) = setup();
        env.set_sender(BOB);
        env.set_timestamp(1500);
        cf.contribute(&env.ctx(), 10_000).unwrap();

        env.set_sender(ALICE);
        env.set_timestamp(2500);
        cf.finalize(&env.ctx()).unwrap();

        env.set_sender(BOB);
        let err = cf.refund(&env.ctx()).unwrap_err();
        assert_err_contains(&err, "refunds only available for failed campaigns");
    }

    #[test]
    fn test_cannot_finalize_before_deadline() {
        let (env, mut cf) = setup();
        env.set_timestamp(1500);
        let err = cf.finalize(&env.ctx()).unwrap_err();
        assert_err_contains(&err, "campaign has not ended yet");
    }

    #[test]
    fn test_multiple_contributors() {
        let (env, mut cf) = setup();
        env.set_timestamp(1500);

        env.set_sender(BOB);
        cf.contribute(&env.ctx(), 3_000).unwrap();
        env.set_sender(ALICE);
        cf.contribute(&env.ctx(), 4_000).unwrap();

        let resp = cf.get_total_raised(&env.ctx()).unwrap();
        let total: u128 = from_response(&resp).unwrap();
        assert_eq!(total, 7_000);

        let resp = cf.get_contributor_count(&env.ctx()).unwrap();
        let count: u64 = from_response(&resp).unwrap();
        assert_eq!(count, 2);
    }
}
