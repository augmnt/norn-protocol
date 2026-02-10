use std::collections::BTreeMap;

use norn_crypto::hash::blake3_hash;
use norn_crypto::keys::verify;
use norn_types::primitives::*;
use norn_types::weave::{StakeOperation, Validator, ValidatorSet};

use crate::error::WeaveError;

/// Per-validator stake tracking.
#[derive(Debug, Clone)]
struct ValidatorStake {
    pubkey: PublicKey,
    address: Address,
    stake: Amount,
    pending_unstake: Option<(Amount, u64)>, // (amount, effective_height)
}

/// Staking state tracking validator stakes, bonding periods, and slashing.
#[derive(Debug, Clone)]
pub struct StakingState {
    validators: BTreeMap<PublicKey, ValidatorStake>,
    bonding_period: u64,
    min_stake: Amount,
}

impl StakingState {
    /// Create a new staking state with the given parameters.
    pub fn new(min_stake: Amount, bonding_period: u64) -> Self {
        Self {
            validators: BTreeMap::new(),
            bonding_period,
            min_stake,
        }
    }

    /// Stake tokens to become or increase stake as a validator.
    pub fn stake(
        &mut self,
        pubkey: PublicKey,
        address: Address,
        amount: Amount,
    ) -> Result<(), WeaveError> {
        if amount == 0 {
            return Err(WeaveError::StakingError {
                reason: "stake amount must be positive".to_string(),
            });
        }

        let entry = self.validators.entry(pubkey).or_insert(ValidatorStake {
            pubkey,
            address,
            stake: 0,
            pending_unstake: None,
        });

        entry.stake = entry.stake.saturating_add(amount);

        if entry.stake < self.min_stake {
            return Err(WeaveError::StakingError {
                reason: format!(
                    "total stake {} below minimum {}",
                    entry.stake, self.min_stake
                ),
            });
        }

        Ok(())
    }

    /// Request to unstake tokens (subject to bonding period).
    pub fn unstake(
        &mut self,
        pubkey: &PublicKey,
        amount: Amount,
        current_height: u64,
    ) -> Result<(), WeaveError> {
        let entry = self
            .validators
            .get_mut(pubkey)
            .ok_or_else(|| WeaveError::StakingError {
                reason: "validator not found".to_string(),
            })?;

        if amount == 0 || amount > entry.stake {
            return Err(WeaveError::StakingError {
                reason: format!(
                    "invalid unstake amount {}: current stake {}",
                    amount, entry.stake
                ),
            });
        }

        let effective_height = current_height + self.bonding_period;
        // Accumulate pending unstakes.
        match &entry.pending_unstake {
            Some((existing_amount, _)) => {
                entry.pending_unstake =
                    Some((existing_amount.saturating_add(amount), effective_height));
            }
            None => {
                entry.pending_unstake = Some((amount, effective_height));
            }
        }

        Ok(())
    }

    /// Slash a validator's stake.
    pub fn slash(&mut self, pubkey: &PublicKey, slash_amount: Amount) -> Result<(), WeaveError> {
        let entry = self
            .validators
            .get_mut(pubkey)
            .ok_or_else(|| WeaveError::StakingError {
                reason: "validator not found".to_string(),
            })?;

        entry.stake = entry.stake.saturating_sub(slash_amount);

        // Also reduce any pending unstake proportionally.
        if let Some((pending, height)) = &entry.pending_unstake {
            let new_pending = pending.saturating_sub(slash_amount);
            if new_pending == 0 {
                entry.pending_unstake = None;
            } else {
                entry.pending_unstake = Some((new_pending, *height));
            }
        }

        Ok(())
    }

    /// Process pending unstakes at the given block height.
    /// Returns the public keys of validators that were removed (stake dropped to zero or below min).
    pub fn process_epoch(&mut self, current_height: u64) -> Vec<PublicKey> {
        let mut removed = Vec::new();

        for (_, entry) in self.validators.iter_mut() {
            if let Some((amount, effective_height)) = entry.pending_unstake {
                if current_height >= effective_height {
                    entry.stake = entry.stake.saturating_sub(amount);
                    entry.pending_unstake = None;
                }
            }
        }

        // Remove validators whose stake dropped below the minimum.
        let keys_to_check: Vec<PublicKey> = self.validators.keys().copied().collect();
        for key in keys_to_check {
            if let Some(entry) = self.validators.get(&key) {
                if entry.stake < self.min_stake {
                    removed.push(key);
                }
            }
        }

        for key in &removed {
            self.validators.remove(key);
        }

        removed
    }

    /// Get the current active validator set, sorted by stake descending.
    pub fn active_validators(&self) -> ValidatorSet {
        let mut validators: Vec<Validator> = self
            .validators
            .values()
            .filter(|v| v.stake >= self.min_stake)
            .map(|v| Validator {
                pubkey: v.pubkey,
                address: v.address,
                stake: v.stake,
                active: true,
            })
            .collect();

        validators.sort_by(|a, b| b.stake.cmp(&a.stake));

        let total_stake: Amount = validators.iter().map(|v| v.stake).sum();

        ValidatorSet {
            validators,
            total_stake,
            epoch: 0,
        }
    }

    /// Check if a public key is an active validator.
    pub fn is_validator(&self, pubkey: &PublicKey) -> bool {
        self.validators
            .get(pubkey)
            .map(|v| v.stake >= self.min_stake)
            .unwrap_or(false)
    }

    /// Get the stake for a validator.
    pub fn validator_stake(&self, pubkey: &PublicKey) -> Option<Amount> {
        self.validators.get(pubkey).map(|v| v.stake)
    }

    /// Get the minimum stake requirement.
    pub fn min_stake(&self) -> Amount {
        self.min_stake
    }

    /// Get the bonding period in blocks.
    pub fn bonding_period(&self) -> u64 {
        self.bonding_period
    }

    /// Get the total staked amount across all validators.
    pub fn total_staked(&self) -> Amount {
        self.validators.values().map(|v| v.stake).sum()
    }
}

/// Compute the signing data for a stake operation.
/// The wallet signs this data to authorize the stake/unstake.
pub fn stake_operation_signing_data(op: &StakeOperation) -> Vec<u8> {
    let mut data = Vec::new();
    match op {
        StakeOperation::Stake {
            pubkey,
            amount,
            timestamp,
            ..
        } => {
            data.extend_from_slice(pubkey);
            data.extend_from_slice(&amount.to_le_bytes());
            data.extend_from_slice(&timestamp.to_le_bytes());
            data.extend_from_slice(b"stake");
        }
        StakeOperation::Unstake {
            pubkey,
            amount,
            timestamp,
            ..
        } => {
            data.extend_from_slice(pubkey);
            data.extend_from_slice(&amount.to_le_bytes());
            data.extend_from_slice(&timestamp.to_le_bytes());
            data.extend_from_slice(b"unstake");
        }
    }
    blake3_hash(&data).to_vec()
}

/// Validate a stake operation: check signature and parameters.
pub fn validate_stake_operation(
    op: &StakeOperation,
    staking: &StakingState,
) -> Result<(), WeaveError> {
    let sig_data = stake_operation_signing_data(op);
    match op {
        StakeOperation::Stake {
            pubkey,
            amount,
            signature,
            ..
        } => {
            verify(&sig_data, signature, pubkey).map_err(|_| WeaveError::StakingError {
                reason: "invalid stake signature".to_string(),
            })?;
            if *amount == 0 {
                return Err(WeaveError::StakingError {
                    reason: "stake amount must be positive".to_string(),
                });
            }
            // For new validators, check amount meets minimum.
            if staking.validator_stake(pubkey).is_none() && *amount < staking.min_stake() {
                return Err(WeaveError::StakingError {
                    reason: format!(
                        "initial stake {} below minimum {}",
                        amount,
                        staking.min_stake()
                    ),
                });
            }
            Ok(())
        }
        StakeOperation::Unstake {
            pubkey,
            amount,
            signature,
            ..
        } => {
            verify(&sig_data, signature, pubkey).map_err(|_| WeaveError::StakingError {
                reason: "invalid unstake signature".to_string(),
            })?;
            if *amount == 0 {
                return Err(WeaveError::StakingError {
                    reason: "unstake amount must be positive".to_string(),
                });
            }
            let current =
                staking
                    .validator_stake(pubkey)
                    .ok_or_else(|| WeaveError::StakingError {
                        reason: "validator not found".to_string(),
                    })?;
            if *amount > current {
                return Err(WeaveError::StakingError {
                    reason: format!("unstake amount {} exceeds stake {}", amount, current),
                });
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pubkey(byte: u8) -> PublicKey {
        [byte; 32]
    }

    fn make_address(byte: u8) -> Address {
        [byte; 20]
    }

    #[test]
    fn test_stake_and_active() {
        let mut staking = StakingState::new(100, 10);
        let pk = make_pubkey(1);
        staking.stake(pk, make_address(1), 500).unwrap();

        assert!(staking.is_validator(&pk));
        assert_eq!(staking.validator_stake(&pk), Some(500));

        let vs = staking.active_validators();
        assert_eq!(vs.len(), 1);
        assert_eq!(vs.validators[0].stake, 500);
    }

    #[test]
    fn test_stake_below_minimum() {
        let mut staking = StakingState::new(100, 10);
        let pk = make_pubkey(1);
        let result = staking.stake(pk, make_address(1), 50);
        assert!(result.is_err());
    }

    #[test]
    fn test_stake_zero_amount() {
        let mut staking = StakingState::new(100, 10);
        let pk = make_pubkey(1);
        let result = staking.stake(pk, make_address(1), 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_unstake() {
        let mut staking = StakingState::new(100, 10);
        let pk = make_pubkey(1);
        staking.stake(pk, make_address(1), 500).unwrap();

        staking.unstake(&pk, 200, 100).unwrap();

        // Validator is still active until bonding period ends.
        assert!(staking.is_validator(&pk));
        assert_eq!(staking.validator_stake(&pk), Some(500));

        // Process epoch before bonding period ends.
        let removed = staking.process_epoch(105);
        assert!(removed.is_empty());
        assert_eq!(staking.validator_stake(&pk), Some(500));

        // Process epoch after bonding period.
        let removed = staking.process_epoch(110);
        assert!(removed.is_empty());
        assert_eq!(staking.validator_stake(&pk), Some(300));
    }

    #[test]
    fn test_unstake_full_removal() {
        let mut staking = StakingState::new(100, 10);
        let pk = make_pubkey(1);
        staking.stake(pk, make_address(1), 500).unwrap();
        staking.unstake(&pk, 500, 100).unwrap();

        let removed = staking.process_epoch(110);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], pk);
        assert!(!staking.is_validator(&pk));
    }

    #[test]
    fn test_slash() {
        let mut staking = StakingState::new(100, 10);
        let pk = make_pubkey(1);
        staking.stake(pk, make_address(1), 500).unwrap();
        staking.slash(&pk, 200).unwrap();

        assert_eq!(staking.validator_stake(&pk), Some(300));
    }

    #[test]
    fn test_slash_below_minimum() {
        let mut staking = StakingState::new(100, 10);
        let pk = make_pubkey(1);
        staking.stake(pk, make_address(1), 150).unwrap();
        staking.slash(&pk, 100).unwrap();

        // Stake is 50, below minimum 100, so not an active validator.
        assert!(!staking.is_validator(&pk));
        let vs = staking.active_validators();
        assert!(vs.is_empty());
    }

    #[test]
    fn test_active_validators_sorted_by_stake() {
        let mut staking = StakingState::new(100, 10);
        staking.stake(make_pubkey(1), make_address(1), 300).unwrap();
        staking.stake(make_pubkey(2), make_address(2), 500).unwrap();
        staking.stake(make_pubkey(3), make_address(3), 100).unwrap();

        let vs = staking.active_validators();
        assert_eq!(vs.len(), 3);
        assert_eq!(vs.validators[0].stake, 500);
        assert_eq!(vs.validators[1].stake, 300);
        assert_eq!(vs.validators[2].stake, 100);
    }

    #[test]
    fn test_unstake_nonexistent_validator() {
        let mut staking = StakingState::new(100, 10);
        let result = staking.unstake(&make_pubkey(99), 100, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_unstake_more_than_staked() {
        let mut staking = StakingState::new(100, 10);
        let pk = make_pubkey(1);
        staking.stake(pk, make_address(1), 500).unwrap();
        let result = staking.unstake(&pk, 600, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_bonding_period() {
        let mut staking = StakingState::new(100, 20);
        let pk = make_pubkey(1);
        staking.stake(pk, make_address(1), 500).unwrap();
        staking.unstake(&pk, 500, 50).unwrap();

        // At height 60, still within bonding period (effective at 70).
        let removed = staking.process_epoch(60);
        assert!(removed.is_empty());
        assert!(staking.is_validator(&pk));

        // At height 70, bonding period complete.
        let removed = staking.process_epoch(70);
        assert_eq!(removed.len(), 1);
    }
}
