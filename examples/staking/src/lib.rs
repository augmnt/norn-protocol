//! Staking Vault — deposit tokens for a lock period, earn rewards.
//! Operator funds the reward pool. Stakers claim proportional rewards.

#![no_std]

extern crate alloc;

use alloc::format;
use norn_sdk::prelude::*;

// ── Storage ────────────────────────────────────────────────────────────

const INITIALIZED: Item<bool> = Item::new("initialized");
const CONFIG: Item<StakingConfig> = Item::new("config");
const TOTAL_STAKED: Item<u128> = Item::new("total_staked");
const REWARD_POOL: Item<u128> = Item::new("reward_pool");
const STAKES: Map<Address, StakeInfo> = Map::new("stakes");

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct StakingConfig {
    pub operator: Address,
    pub token_id: TokenId,
    pub reward_rate: u128,   // reward per second per 1e12 staked
    pub min_lock_period: u64,
    pub created_at: u64,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct StakeInfo {
    pub amount: u128,
    pub start_time: u64,
    pub last_claim_time: u64,
}

// ── Reward math ────────────────────────────────────────────────────────

fn calculate_pending(stake: &StakeInfo, config: &StakingConfig, now: u64) -> Result<u128, ContractError> {
    if stake.amount == 0 {
        return Ok(0);
    }
    let elapsed = if now > stake.last_claim_time {
        now - stake.last_claim_time
    } else {
        0
    };
    // rewards = stake.amount * elapsed * reward_rate / 1e12
    let product = safe_mul(stake.amount, elapsed as u128)?;
    let scaled = safe_mul(product, config.reward_rate)?;
    Ok(scaled / 1_000_000_000_000)
}

// ── Contract ───────────────────────────────────────────────────────────

#[norn_contract]
pub struct Staking;

#[norn_contract]
impl Staking {
    #[init]
    pub fn new(_ctx: &Context) -> Self {
        INITIALIZED.init(&false);
        TOTAL_STAKED.init(&0u128);
        REWARD_POOL.init(&0u128);
        Staking
    }

    #[execute]
    pub fn initialize(
        &mut self,
        ctx: &Context,
        token_id: TokenId,
        reward_rate: u128,
        min_lock_period: u64,
    ) -> ContractResult {
        ensure!(!INITIALIZED.load_or(false), "already initialized");
        ensure!(reward_rate > 0, "reward_rate must be positive");

        CONFIG.save(&StakingConfig {
            operator: ctx.sender(),
            token_id,
            reward_rate,
            min_lock_period,
            created_at: ctx.timestamp(),
        })?;
        INITIALIZED.save(&true)?;

        Ok(Response::with_action("initialize"))
    }

    #[execute]
    pub fn stake(&mut self, ctx: &Context, amount: u128) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(amount > 0, "amount must be positive");

        let contract = ctx.contract_address();
        ctx.transfer(&ctx.sender(), &contract, &config.token_id, amount);

        let mut info = STAKES.load(&ctx.sender()).unwrap_or(StakeInfo {
            amount: 0,
            start_time: ctx.timestamp(),
            last_claim_time: ctx.timestamp(),
        });

        // If existing stake, auto-claim pending rewards first
        if info.amount > 0 {
            let pending = calculate_pending(&info, &config, ctx.timestamp())?;
            let pool = REWARD_POOL.load_or(0u128);
            let claimable = if pending > pool { pool } else { pending };
            if claimable > 0 {
                ctx.transfer_from_contract(&ctx.sender(), &config.token_id, claimable);
                REWARD_POOL.save(&safe_sub(pool, claimable)?)?;
            }
        }

        info.amount = safe_add(info.amount, amount)?;
        info.last_claim_time = ctx.timestamp();
        STAKES.save(&ctx.sender(), &info)?;

        let total = TOTAL_STAKED.load_or(0u128);
        TOTAL_STAKED.save(&safe_add(total, amount)?)?;

        Ok(Response::with_action("stake")
            .add_attribute("amount", format!("{}", amount)))
    }

    #[execute]
    pub fn unstake(&mut self, ctx: &Context, amount: u128) -> ContractResult {
        let config = CONFIG.load()?;
        let mut info = STAKES.load(&ctx.sender())?;
        ensure!(amount > 0, "amount must be positive");
        ensure!(info.amount >= amount, "insufficient stake");

        let elapsed = if ctx.timestamp() > info.start_time {
            ctx.timestamp() - info.start_time
        } else {
            0
        };
        ensure!(
            elapsed >= config.min_lock_period,
            "lock period has not ended"
        );

        // Auto-claim pending rewards
        let pending = calculate_pending(&info, &config, ctx.timestamp())?;
        let pool = REWARD_POOL.load_or(0u128);
        let claimable = if pending > pool { pool } else { pending };
        if claimable > 0 {
            ctx.transfer_from_contract(&ctx.sender(), &config.token_id, claimable);
            REWARD_POOL.save(&safe_sub(pool, claimable)?)?;
        }

        // Return staked tokens
        ctx.transfer_from_contract(&ctx.sender(), &config.token_id, amount);

        info.amount = safe_sub(info.amount, amount)?;
        info.last_claim_time = ctx.timestamp();
        STAKES.save(&ctx.sender(), &info)?;

        let total = TOTAL_STAKED.load_or(0u128);
        TOTAL_STAKED.save(&safe_sub(total, amount)?)?;

        Ok(Response::with_action("unstake")
            .add_attribute("amount", format!("{}", amount)))
    }

    #[execute]
    pub fn claim_rewards(&mut self, ctx: &Context) -> ContractResult {
        let config = CONFIG.load()?;
        let mut info = STAKES.load(&ctx.sender())?;
        ensure!(info.amount > 0, "no active stake");

        let pending = calculate_pending(&info, &config, ctx.timestamp())?;
        let pool = REWARD_POOL.load_or(0u128);
        let claimable = if pending > pool { pool } else { pending };
        ensure!(claimable > 0, "no rewards to claim");

        ctx.transfer_from_contract(&ctx.sender(), &config.token_id, claimable);
        REWARD_POOL.save(&safe_sub(pool, claimable)?)?;

        info.last_claim_time = ctx.timestamp();
        STAKES.save(&ctx.sender(), &info)?;

        Ok(Response::with_action("claim_rewards")
            .add_attribute("amount", format!("{}", claimable)))
    }

    #[execute]
    pub fn fund_rewards(&mut self, ctx: &Context, amount: u128) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(amount > 0, "amount must be positive");

        let contract = ctx.contract_address();
        ctx.transfer(&ctx.sender(), &contract, &config.token_id, amount);

        let pool = REWARD_POOL.load_or(0u128);
        REWARD_POOL.save(&safe_add(pool, amount)?)?;

        Ok(Response::with_action("fund_rewards")
            .add_attribute("amount", format!("{}", amount)))
    }

    #[query]
    pub fn get_config(&self, _ctx: &Context) -> ContractResult {
        let config = CONFIG.load()?;
        ok(config)
    }

    #[query]
    pub fn get_stake(&self, _ctx: &Context, addr: Address) -> ContractResult {
        let info = STAKES.load(&addr).unwrap_or(StakeInfo {
            amount: 0,
            start_time: 0,
            last_claim_time: 0,
        });
        ok(info)
    }

    #[query]
    pub fn get_pending_rewards(&self, ctx: &Context, addr: Address) -> ContractResult {
        let config = CONFIG.load()?;
        let info = STAKES.load(&addr).unwrap_or(StakeInfo {
            amount: 0,
            start_time: 0,
            last_claim_time: 0,
        });
        let pending = calculate_pending(&info, &config, ctx.timestamp())?;
        let pool = REWARD_POOL.load_or(0u128);
        let claimable = if pending > pool { pool } else { pending };
        ok(claimable)
    }

    #[query]
    pub fn get_total_staked(&self, _ctx: &Context) -> ContractResult {
        let total = TOTAL_STAKED.load_or(0u128);
        ok(total)
    }

    #[query]
    pub fn get_reward_pool(&self, _ctx: &Context) -> ContractResult {
        let pool = REWARD_POOL.load_or(0u128);
        ok(pool)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    const TOKEN: TokenId = [42u8; 32];
    const CONTRACT_ADDR: Address = [99u8; 20];

    fn setup() -> (TestEnv, Staking) {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let mut st = Staking::new(&env.ctx());
        st.initialize(&env.ctx(), TOKEN, 1_000_000, 100) // 1e6 rate, 100s lock
            .unwrap();
        // Fund reward pool generously
        st.fund_rewards(&env.ctx(), 1_000_000_000).unwrap();
        (env, st)
    }

    #[test]
    fn test_initialize() {
        let (env, st) = setup();
        let resp = st.get_config(&env.ctx()).unwrap();
        let config: StakingConfig = from_response(&resp).unwrap();
        assert_eq!(config.reward_rate, 1_000_000);
        assert_eq!(config.min_lock_period, 100);
    }

    #[test]
    fn test_stake() {
        let (env, mut st) = setup();
        env.set_sender(BOB);
        st.stake(&env.ctx(), 5_000).unwrap();

        let resp = st.get_stake(&env.ctx(), BOB).unwrap();
        let info: StakeInfo = from_response(&resp).unwrap();
        assert_eq!(info.amount, 5_000);

        let resp = st.get_total_staked(&env.ctx()).unwrap();
        let total: u128 = from_response(&resp).unwrap();
        assert_eq!(total, 5_000);
    }

    #[test]
    fn test_pending_rewards() {
        let (env, mut st) = setup();
        env.set_sender(BOB);
        st.stake(&env.ctx(), 1_000_000_000_000).unwrap(); // 1e12

        // After 100 seconds: rewards = 1e12 * 100 * 1e6 / 1e12 = 100_000_000
        env.set_timestamp(1100);
        let resp = st.get_pending_rewards(&env.ctx(), BOB).unwrap();
        let pending: u128 = from_response(&resp).unwrap();
        assert_eq!(pending, 100_000_000);
    }

    #[test]
    fn test_claim_rewards() {
        let (env, mut st) = setup();
        env.set_sender(BOB);
        st.stake(&env.ctx(), 1_000_000_000_000).unwrap();

        env.set_timestamp(1100);
        st.claim_rewards(&env.ctx()).unwrap();

        // Verify claim resets pending
        let resp = st.get_pending_rewards(&env.ctx(), BOB).unwrap();
        let pending: u128 = from_response(&resp).unwrap();
        assert_eq!(pending, 0);
    }

    #[test]
    fn test_unstake_after_lock() {
        let (env, mut st) = setup();
        env.set_sender(BOB);
        st.stake(&env.ctx(), 5_000).unwrap();

        env.set_timestamp(1100); // 100s elapsed, lock met
        st.unstake(&env.ctx(), 5_000).unwrap();

        let resp = st.get_total_staked(&env.ctx()).unwrap();
        let total: u128 = from_response(&resp).unwrap();
        assert_eq!(total, 0);
    }

    #[test]
    fn test_cannot_unstake_before_lock() {
        let (env, mut st) = setup();
        env.set_sender(BOB);
        st.stake(&env.ctx(), 5_000).unwrap();

        env.set_timestamp(1050); // only 50s, need 100
        let err = st.unstake(&env.ctx(), 5_000).unwrap_err();
        assert_err_contains(&err, "lock period has not ended");
    }

    #[test]
    fn test_fund_rewards() {
        let (env, mut st) = setup();
        st.fund_rewards(&env.ctx(), 50_000).unwrap();

        let resp = st.get_reward_pool(&env.ctx()).unwrap();
        let pool: u128 = from_response(&resp).unwrap();
        assert_eq!(pool, 1_000_050_000); // initial 1B + 50K
    }

    #[test]
    fn test_rewards_capped_by_pool() {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let mut st = Staking::new(&env.ctx());
        st.initialize(&env.ctx(), TOKEN, 1_000_000, 0).unwrap();
        // Fund only 10 tokens
        st.fund_rewards(&env.ctx(), 10).unwrap();

        env.set_sender(BOB);
        st.stake(&env.ctx(), 1_000_000_000_000).unwrap();

        // After long time, rewards would be huge but capped by pool
        env.set_timestamp(2000);
        let resp = st.get_pending_rewards(&env.ctx(), BOB).unwrap();
        let pending: u128 = from_response(&resp).unwrap();
        assert_eq!(pending, 10); // capped at pool size
    }
}
