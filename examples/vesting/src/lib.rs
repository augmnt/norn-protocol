//! Token Vesting contract — time-locked token releases with cliff periods.
//! Anyone can create vesting schedules. Beneficiaries claim tokens as they
//! vest. Creators can revoke revocable schedules.

#![no_std]

extern crate alloc;

use norn_sdk::prelude::*;

// ── Storage layout ──────────────────────────────────────────────────────

const SCHEDULE_COUNT: Item<u64> = Item::new("schedule_count");
const SCHEDULES: Map<u64, VestingSchedule> = Map::new("schedules");

// ── Types ───────────────────────────────────────────────────────────────

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct VestingSchedule {
    pub id: u64,
    pub creator: Address,
    pub beneficiary: Address,
    pub token_id: TokenId,
    pub total_amount: u128,
    pub claimed_amount: u128,
    pub start_time: u64,
    pub cliff_duration: u64,
    pub total_duration: u64,
    pub revocable: bool,
    pub revoked: bool,
    pub created_at: u64,
}

// ── Vesting math ────────────────────────────────────────────────────────

fn calculate_vested(schedule: &VestingSchedule, now: u64) -> Result<u128, ContractError> {
    if now < schedule.start_time {
        return Ok(0);
    }
    let elapsed = now - schedule.start_time;
    if elapsed < schedule.cliff_duration {
        return Ok(0);
    }
    if elapsed >= schedule.total_duration {
        return Ok(schedule.total_amount);
    }
    // (total_amount * elapsed) / total_duration — safe math
    let product = safe_mul(schedule.total_amount, elapsed as u128)?;
    Ok(product / (schedule.total_duration as u128))
}

// ── Contract ────────────────────────────────────────────────────────────

#[norn_contract]
pub struct Vesting;

#[norn_contract]
impl Vesting {
    #[init]
    pub fn new(_ctx: &Context) -> Self {
        SCHEDULE_COUNT.init(&0u64);
        Vesting
    }

    #[execute]
    #[allow(clippy::too_many_arguments)]
    pub fn create_schedule(
        &mut self,
        ctx: &Context,
        beneficiary: Address,
        token_id: TokenId,
        amount: u128,
        start_time: u64,
        cliff_duration: u64,
        total_duration: u64,
        revocable: bool,
    ) -> ContractResult {
        ensure!(amount > 0, "amount must be positive");
        ensure!(total_duration > 0, "total_duration must be positive");
        ensure!(
            cliff_duration <= total_duration,
            "cliff_duration exceeds total_duration"
        );
        ensure!(
            beneficiary != ZERO_ADDRESS,
            "beneficiary cannot be zero address"
        );

        // Transfer tokens from creator to contract
        let contract = ctx.contract_address();
        ctx.transfer(&ctx.sender(), &contract, &token_id, amount);

        let id = SCHEDULE_COUNT.load_or(0u64);
        let schedule = VestingSchedule {
            id,
            creator: ctx.sender(),
            beneficiary,
            token_id,
            total_amount: amount,
            claimed_amount: 0,
            start_time,
            cliff_duration,
            total_duration,
            revocable,
            revoked: false,
            created_at: ctx.timestamp(),
        };
        SCHEDULES.save(&id, &schedule)?;
        SCHEDULE_COUNT.save(&safe_add_u64(id, 1)?)?;

        Ok(Response::with_action("create_schedule")
            .add_attribute("schedule_id", format!("{}", id))
            .set_data(&id))
    }

    #[execute]
    pub fn claim(&mut self, ctx: &Context, schedule_id: u64) -> ContractResult {
        let mut schedule = SCHEDULES.load(&schedule_id)?;
        ensure!(
            schedule.beneficiary == ctx.sender(),
            "only beneficiary can claim"
        );
        ensure!(!schedule.revoked, "schedule has been revoked");

        let vested = calculate_vested(&schedule, ctx.timestamp())?;
        let claimable = safe_sub(vested, schedule.claimed_amount)?;
        ensure!(claimable > 0, "nothing to claim");

        // Transfer from contract to beneficiary
        ctx.transfer_from_contract(&schedule.beneficiary, &schedule.token_id, claimable);

        schedule.claimed_amount = safe_add(schedule.claimed_amount, claimable)?;
        SCHEDULES.save(&schedule_id, &schedule)?;

        Ok(Response::with_action("claim")
            .add_attribute("schedule_id", format!("{}", schedule_id))
            .add_attribute("claimed", format!("{}", claimable)))
    }

    #[execute]
    pub fn revoke(&mut self, ctx: &Context, schedule_id: u64) -> ContractResult {
        let mut schedule = SCHEDULES.load(&schedule_id)?;
        ensure!(
            schedule.creator == ctx.sender(),
            "only creator can revoke"
        );
        ensure!(schedule.revocable, "schedule is not revocable");
        ensure!(!schedule.revoked, "schedule already revoked");

        // Calculate how much is vested but unclaimed — send to beneficiary
        let vested = calculate_vested(&schedule, ctx.timestamp())?;
        let unclaimed_vested = safe_sub(vested, schedule.claimed_amount)?;

        if unclaimed_vested > 0 {
            ctx.transfer_from_contract(
                &schedule.beneficiary,
                &schedule.token_id,
                unclaimed_vested,
            );
        }

        // Send unvested back to creator
        let unvested = safe_sub(schedule.total_amount, vested)?;
        if unvested > 0 {
            ctx.transfer_from_contract(&schedule.creator, &schedule.token_id, unvested);
        }

        schedule.revoked = true;
        schedule.claimed_amount = vested;
        SCHEDULES.save(&schedule_id, &schedule)?;

        Ok(Response::with_action("revoke")
            .add_attribute("schedule_id", format!("{}", schedule_id))
            .add_attribute("returned_to_beneficiary", format!("{}", unclaimed_vested))
            .add_attribute("returned_to_creator", format!("{}", unvested)))
    }

    #[query]
    pub fn get_schedule(&self, _ctx: &Context, schedule_id: u64) -> ContractResult {
        let schedule = SCHEDULES.load(&schedule_id)?;
        ok(schedule)
    }

    #[query]
    pub fn get_schedule_count(&self, _ctx: &Context) -> ContractResult {
        let count = SCHEDULE_COUNT.load_or(0u64);
        ok(count)
    }

    #[query]
    pub fn get_claimable(&self, ctx: &Context, schedule_id: u64) -> ContractResult {
        let schedule = SCHEDULES.load(&schedule_id)?;
        if schedule.revoked {
            return ok(0u128);
        }
        let vested = calculate_vested(&schedule, ctx.timestamp())?;
        let claimable = safe_sub(vested, schedule.claimed_amount)?;
        ok(claimable)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    const TOKEN: TokenId = [42u8; 32];
    const CONTRACT_ADDR: Address = [99u8; 20];

    fn setup() -> (TestEnv, Vesting) {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let vesting = Vesting::new(&env.ctx());
        (env, vesting)
    }

    /// Create a standard schedule: 10_000 tokens, starts at t=1000,
    /// cliff=100s, total=1000s, revocable.
    fn create_standard_schedule(env: &TestEnv, vesting: &mut Vesting) -> u64 {
        let resp = vesting
            .create_schedule(
                &env.ctx(),
                BOB,     // beneficiary
                TOKEN,
                10_000,  // amount
                1000,    // start_time
                100,     // cliff_duration
                1000,    // total_duration
                true,    // revocable
            )
            .unwrap();
        from_response::<u64>(&resp).unwrap()
    }

    #[test]
    fn test_create_schedule() {
        let (env, mut vesting) = setup();
        let id = create_standard_schedule(&env, &mut vesting);
        assert_eq!(id, 0);

        let resp = vesting.get_schedule(&env.ctx(), 0).unwrap();
        let s: VestingSchedule = from_response(&resp).unwrap();
        assert_eq!(s.creator, ALICE);
        assert_eq!(s.beneficiary, BOB);
        assert_eq!(s.total_amount, 10_000);
        assert_eq!(s.claimed_amount, 0);
        assert!(s.revocable);
        assert!(!s.revoked);

        // Verify deposit transfer: ALICE -> contract
        let transfers = env.transfers();
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].0, ALICE.to_vec());
        assert_eq!(transfers[0].1, CONTRACT_ADDR.to_vec());
        assert_eq!(transfers[0].3, 10_000);
    }

    #[test]
    fn test_cannot_claim_before_cliff() {
        let (env, mut vesting) = setup();
        create_standard_schedule(&env, &mut vesting);

        // t=1050, cliff ends at t=1100
        env.set_timestamp(1050);
        env.set_sender(BOB);

        let err = vesting.claim(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "nothing to claim");
    }

    #[test]
    fn test_claim_after_cliff() {
        let (env, mut vesting) = setup();
        create_standard_schedule(&env, &mut vesting);

        // t=1500 → 500/1000 elapsed → 50% vested = 5000
        env.set_timestamp(1500);
        env.set_sender(BOB);

        vesting.claim(&env.ctx(), 0).unwrap();

        let resp = vesting.get_schedule(&env.ctx(), 0).unwrap();
        let s: VestingSchedule = from_response(&resp).unwrap();
        assert_eq!(s.claimed_amount, 5000);

        // Verify transfer: contract -> BOB
        let transfers = env.transfers();
        assert_eq!(transfers.len(), 2); // deposit + claim
        assert_eq!(transfers[1].0, CONTRACT_ADDR.to_vec());
        assert_eq!(transfers[1].1, BOB.to_vec());
        assert_eq!(transfers[1].3, 5000);
    }

    #[test]
    fn test_claim_partial_vesting() {
        let (env, mut vesting) = setup();
        create_standard_schedule(&env, &mut vesting);

        // t=1200 → 200/1000 elapsed → 20% vested = 2000
        env.set_timestamp(1200);
        env.set_sender(BOB);

        let resp = vesting.get_claimable(&env.ctx(), 0).unwrap();
        let claimable: u128 = from_response(&resp).unwrap();
        assert_eq!(claimable, 2000);

        vesting.claim(&env.ctx(), 0).unwrap();

        let resp = vesting.get_schedule(&env.ctx(), 0).unwrap();
        let s: VestingSchedule = from_response(&resp).unwrap();
        assert_eq!(s.claimed_amount, 2000);
    }

    #[test]
    fn test_claim_full_vesting() {
        let (env, mut vesting) = setup();
        create_standard_schedule(&env, &mut vesting);

        // t=2000 → past total_duration → 100% vested
        env.set_timestamp(2000);
        env.set_sender(BOB);

        vesting.claim(&env.ctx(), 0).unwrap();

        let resp = vesting.get_schedule(&env.ctx(), 0).unwrap();
        let s: VestingSchedule = from_response(&resp).unwrap();
        assert_eq!(s.claimed_amount, 10_000);
    }

    #[test]
    fn test_revoke_revocable() {
        let (env, mut vesting) = setup();
        create_standard_schedule(&env, &mut vesting);

        // t=1500 → 50% vested = 5000
        env.set_timestamp(1500);

        // Creator (ALICE) revokes
        vesting.revoke(&env.ctx(), 0).unwrap();

        let resp = vesting.get_schedule(&env.ctx(), 0).unwrap();
        let s: VestingSchedule = from_response(&resp).unwrap();
        assert!(s.revoked);

        // Transfers: deposit + vested_to_beneficiary + unvested_to_creator
        let transfers = env.transfers();
        assert_eq!(transfers.len(), 3);
        // Vested unclaimed (5000) -> BOB
        assert_eq!(transfers[1].0, CONTRACT_ADDR.to_vec());
        assert_eq!(transfers[1].1, BOB.to_vec());
        assert_eq!(transfers[1].3, 5000);
        // Unvested (5000) -> ALICE
        assert_eq!(transfers[2].0, CONTRACT_ADDR.to_vec());
        assert_eq!(transfers[2].1, ALICE.to_vec());
        assert_eq!(transfers[2].3, 5000);
    }

    #[test]
    fn test_cannot_revoke_non_revocable() {
        let (env, mut vesting) = setup();

        // Create non-revocable schedule
        vesting
            .create_schedule(&env.ctx(), BOB, TOKEN, 10_000, 1000, 100, 1000, false)
            .unwrap();

        let err = vesting.revoke(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "schedule is not revocable");
    }

    #[test]
    fn test_cannot_revoke_already_revoked() {
        let (env, mut vesting) = setup();
        create_standard_schedule(&env, &mut vesting);

        env.set_timestamp(1500);
        vesting.revoke(&env.ctx(), 0).unwrap();

        let err = vesting.revoke(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "schedule already revoked");
    }

    #[test]
    fn test_only_beneficiary_can_claim() {
        let (env, mut vesting) = setup();
        create_standard_schedule(&env, &mut vesting);

        env.set_timestamp(1500);
        // ALICE (creator) tries to claim — should fail
        let err = vesting.claim(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "only beneficiary can claim");
    }

    #[test]
    fn test_only_creator_can_revoke() {
        let (env, mut vesting) = setup();
        create_standard_schedule(&env, &mut vesting);

        env.set_timestamp(1500);
        env.set_sender(BOB);

        let err = vesting.revoke(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "only creator can revoke");
    }

    #[test]
    fn test_vesting_math_large_amounts() {
        let (env, mut vesting) = setup();

        // Large amount to test precision: 1_000_000_000_000 tokens
        let large_amount: u128 = 1_000_000_000_000;
        vesting
            .create_schedule(&env.ctx(), BOB, TOKEN, large_amount, 1000, 0, 1_000_000, false)
            .unwrap();

        // 33.33% elapsed
        env.set_timestamp(1000 + 333_333);
        env.set_sender(BOB);

        let resp = vesting.get_claimable(&env.ctx(), 0).unwrap();
        let claimable: u128 = from_response(&resp).unwrap();
        // 1_000_000_000_000 * 333_333 / 1_000_000 = 333_333_000_000
        assert_eq!(claimable, 333_333_000_000);
    }

    #[test]
    fn test_multiple_claims_over_time() {
        let (env, mut vesting) = setup();
        create_standard_schedule(&env, &mut vesting);

        env.set_sender(BOB);

        // First claim at t=1200 → 20% = 2000
        env.set_timestamp(1200);
        vesting.claim(&env.ctx(), 0).unwrap();

        let resp = vesting.get_schedule(&env.ctx(), 0).unwrap();
        let s: VestingSchedule = from_response(&resp).unwrap();
        assert_eq!(s.claimed_amount, 2000);

        // Second claim at t=1500 → 50% total = 5000, already claimed 2000, so 3000 more
        env.set_timestamp(1500);
        vesting.claim(&env.ctx(), 0).unwrap();

        let resp = vesting.get_schedule(&env.ctx(), 0).unwrap();
        let s: VestingSchedule = from_response(&resp).unwrap();
        assert_eq!(s.claimed_amount, 5000);

        // Final claim at t=2000 → 100% = 10000, already claimed 5000
        env.set_timestamp(2000);
        vesting.claim(&env.ctx(), 0).unwrap();

        let resp = vesting.get_schedule(&env.ctx(), 0).unwrap();
        let s: VestingSchedule = from_response(&resp).unwrap();
        assert_eq!(s.claimed_amount, 10_000);
    }

    #[test]
    fn test_schedule_count() {
        let (env, mut vesting) = setup();
        create_standard_schedule(&env, &mut vesting);
        create_standard_schedule(&env, &mut vesting);

        let resp = vesting.get_schedule_count(&env.ctx()).unwrap();
        let count: u64 = from_response(&resp).unwrap();
        assert_eq!(count, 2);
    }
}
