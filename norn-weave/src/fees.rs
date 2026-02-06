use norn_types::primitives::Amount;
use norn_types::weave::FeeState;

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
    if 2 * utilized > capacity {
        fee_state.fee_multiplier += fee_state.fee_multiplier / 8;
    } else if 2 * utilized < capacity {
        fee_state.fee_multiplier = fee_state
            .fee_multiplier
            .saturating_sub(fee_state.fee_multiplier / 8);
    }
    fee_state.fee_multiplier = fee_state.fee_multiplier.clamp(100, 10000);
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
}
