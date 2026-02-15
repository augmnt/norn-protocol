use norn_types::primitives::{Address, Amount};
use norn_types::weave::{FeeState, ValidatorSet};

/// Compute the fee for a given number of commitments.
///
/// fee = base_fee * fee_multiplier / 1000 * commitment_count
pub fn compute_fee(fee_state: &FeeState, commitment_count: u64) -> Amount {
    let fee_per = fee_state
        .base_fee
        .saturating_mul(fee_state.fee_multiplier as u128)
        / 1000u128;
    fee_per.saturating_mul(commitment_count as u128)
}

/// Update the fee state based on block utilization.
///
/// Uses integer arithmetic: `2 * utilized > capacity` means > 50% full.
/// If utilization > 50%: increase fee_multiplier by 12.5%.
/// If utilization < 50%: decrease fee_multiplier by 12.5%.
/// Clamp fee_multiplier to [100, 10000].
pub fn update_fee_state(fee_state: &mut FeeState, utilized: u64, capacity: u64) {
    if capacity == 0 {
        return;
    }
    if utilized.saturating_mul(2) > capacity {
        fee_state.fee_multiplier = fee_state
            .fee_multiplier
            .saturating_add(fee_state.fee_multiplier / 8);
    } else if utilized.saturating_mul(2) < capacity {
        fee_state.fee_multiplier = fee_state
            .fee_multiplier
            .saturating_sub(fee_state.fee_multiplier / 8);
    }
    fee_state.fee_multiplier = fee_state.fee_multiplier.clamp(100, 10000);
}

/// Compute the reward distribution for validators based on their stake proportions.
///
/// Each validator receives `total_rewards * validator.stake / total_stake`.
/// Any dust remainder (from integer division) is assigned to the first validator
/// (highest-staked, since ValidatorSet is sorted descending by stake).
///
/// Returns an empty vec if there are no active validators, zero total stake, or zero rewards.
pub fn compute_reward_distribution(
    validator_set: &ValidatorSet,
    total_rewards: Amount,
) -> Vec<(Address, Amount)> {
    if total_rewards == 0 || validator_set.total_stake == 0 || validator_set.validators.is_empty() {
        return vec![];
    }

    let total_stake = validator_set.total_stake;
    let mut distributed: Amount = 0;
    let mut rewards: Vec<(Address, Amount)> = validator_set
        .validators
        .iter()
        .filter(|v| v.active)
        .map(|v| {
            let share = total_rewards
                .saturating_mul(v.stake)
                .checked_div(total_stake)
                .unwrap_or(0);
            distributed = distributed.saturating_add(share);
            (v.address, share)
        })
        .collect();

    // Assign dust remainder to first validator (highest stake).
    if !rewards.is_empty() {
        let dust = total_rewards.saturating_sub(distributed);
        if dust > 0 {
            rewards[0].1 = rewards[0].1.saturating_add(dust);
        }
    }

    rewards
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fee_state(base_fee: Amount, multiplier: u64) -> FeeState {
        FeeState {
            base_fee,
            fee_multiplier: multiplier,
            epoch_fees: 0,
        }
    }

    #[test]
    fn test_compute_fee_basic() {
        let fs = make_fee_state(100, 1000); // 1.0x multiplier
        let fee = compute_fee(&fs, 10);
        // 100 * 1000 / 1000 * 10 = 1000
        assert_eq!(fee, 1000);
    }

    #[test]
    fn test_compute_fee_with_multiplier() {
        let fs = make_fee_state(100, 2000); // 2.0x multiplier
        let fee = compute_fee(&fs, 5);
        // 100 * 2000 / 1000 * 5 = 200 * 5 = 1000
        assert_eq!(fee, 1000);
    }

    #[test]
    fn test_compute_fee_zero_commitments() {
        let fs = make_fee_state(100, 1000);
        let fee = compute_fee(&fs, 0);
        assert_eq!(fee, 0);
    }

    #[test]
    fn test_update_fee_state_increase() {
        let mut fs = make_fee_state(100, 1000);
        update_fee_state(&mut fs, 80, 100); // 80/100 > 50%
                                            // 1000 + 1000/8 = 1000 + 125 = 1125
        assert_eq!(fs.fee_multiplier, 1125);
    }

    #[test]
    fn test_update_fee_state_decrease() {
        let mut fs = make_fee_state(100, 1000);
        update_fee_state(&mut fs, 20, 100); // 20/100 < 50%
                                            // 1000 - 1000/8 = 1000 - 125 = 875
        assert_eq!(fs.fee_multiplier, 875);
    }

    #[test]
    fn test_update_fee_state_at_half() {
        let mut fs = make_fee_state(100, 1000);
        update_fee_state(&mut fs, 50, 100); // exactly 50% => no change
        assert_eq!(fs.fee_multiplier, 1000);
    }

    #[test]
    fn test_fee_multiplier_clamped_min() {
        let mut fs = make_fee_state(100, 100);
        update_fee_state(&mut fs, 10, 100); // 10/100 < 50%
                                            // 100 - 100/8 = 100 - 12 = 88, clamped to 100
        assert_eq!(fs.fee_multiplier, 100);
    }

    #[test]
    fn test_fee_multiplier_clamped_max() {
        let mut fs = make_fee_state(100, 10000);
        update_fee_state(&mut fs, 90, 100); // 90/100 > 50%
                                            // 10000 + 10000/8 = 10000 + 1250 = 11250, clamped to 10000
        assert_eq!(fs.fee_multiplier, 10000);
    }

    #[test]
    fn test_update_fee_state_zero_capacity() {
        let mut fs = make_fee_state(100, 1000);
        update_fee_state(&mut fs, 0, 0); // zero capacity => no change
        assert_eq!(fs.fee_multiplier, 1000);
    }

    // ─── Reward Distribution Tests ─────────────────────────────────────

    use norn_types::weave::{Validator, ValidatorSet};

    fn make_validator(addr_byte: u8, stake: Amount) -> Validator {
        let mut address = [0u8; 20];
        address[19] = addr_byte;
        Validator {
            pubkey: [addr_byte; 32],
            address,
            stake,
            active: true,
        }
    }

    fn make_vs(validators: Vec<Validator>) -> ValidatorSet {
        let total_stake: Amount = validators.iter().map(|v| v.stake).sum();
        ValidatorSet {
            validators,
            total_stake,
            epoch: 0,
        }
    }

    #[test]
    fn test_reward_distribution_proportional() {
        // Two validators: 75% and 25% stake
        let vs = make_vs(vec![make_validator(1, 750), make_validator(2, 250)]);
        let rewards = compute_reward_distribution(&vs, 1000);
        assert_eq!(rewards.len(), 2);
        assert_eq!(rewards[0].1, 750); // 1000 * 750 / 1000
        assert_eq!(rewards[1].1, 250); // 1000 * 250 / 1000
    }

    #[test]
    fn test_reward_distribution_single_validator() {
        let vs = make_vs(vec![make_validator(1, 500)]);
        let rewards = compute_reward_distribution(&vs, 999);
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].1, 999); // gets everything
    }

    #[test]
    fn test_reward_distribution_zero_fees() {
        let vs = make_vs(vec![make_validator(1, 500)]);
        let rewards = compute_reward_distribution(&vs, 0);
        assert!(rewards.is_empty());
    }

    #[test]
    fn test_reward_distribution_no_validators() {
        let vs = ValidatorSet {
            validators: vec![],
            total_stake: 0,
            epoch: 0,
        };
        let rewards = compute_reward_distribution(&vs, 1000);
        assert!(rewards.is_empty());
    }

    #[test]
    fn test_reward_distribution_dust_remainder() {
        // Three validators with equal stake: 1000 / 3 = 333 each, remainder = 1
        let vs = make_vs(vec![
            make_validator(1, 100),
            make_validator(2, 100),
            make_validator(3, 100),
        ]);
        let rewards = compute_reward_distribution(&vs, 1000);
        assert_eq!(rewards.len(), 3);
        // First validator gets dust: 333 + 1 = 334
        assert_eq!(rewards[0].1, 334);
        assert_eq!(rewards[1].1, 333);
        assert_eq!(rewards[2].1, 333);
        // Total should equal input
        let total: Amount = rewards.iter().map(|r| r.1).sum();
        assert_eq!(total, 1000);
    }
}
