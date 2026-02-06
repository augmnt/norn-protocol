use norn_types::primitives::PublicKey;

/// Round-robin leader rotation.
#[derive(Debug, Clone)]
pub struct LeaderRotation {
    validators: Vec<PublicKey>,
}

impl LeaderRotation {
    /// Create a new leader rotation with the given ordered validator list.
    pub fn new(validators: Vec<PublicKey>) -> Self {
        Self { validators }
    }

    /// Get the leader for a given view (round-robin).
    pub fn leader_for_view(&self, view: u64) -> Option<&PublicKey> {
        if self.validators.is_empty() {
            return None;
        }
        let index = (view as usize) % self.validators.len();
        Some(&self.validators[index])
    }

    /// Check if a given public key is the leader for the specified view.
    pub fn is_leader(&self, view: u64, pubkey: &PublicKey) -> bool {
        self.leader_for_view(view)
            .map(|leader| leader == pubkey)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pubkey(byte: u8) -> PublicKey {
        [byte; 32]
    }

    #[test]
    fn test_round_robin() {
        let validators = vec![make_pubkey(1), make_pubkey(2), make_pubkey(3)];
        let rotation = LeaderRotation::new(validators.clone());

        assert_eq!(rotation.leader_for_view(0), Some(&validators[0]));
        assert_eq!(rotation.leader_for_view(1), Some(&validators[1]));
        assert_eq!(rotation.leader_for_view(2), Some(&validators[2]));
        assert_eq!(rotation.leader_for_view(3), Some(&validators[0]));
        assert_eq!(rotation.leader_for_view(4), Some(&validators[1]));
    }

    #[test]
    fn test_is_leader() {
        let validators = vec![make_pubkey(1), make_pubkey(2)];
        let rotation = LeaderRotation::new(validators);

        assert!(rotation.is_leader(0, &make_pubkey(1)));
        assert!(!rotation.is_leader(0, &make_pubkey(2)));
        assert!(rotation.is_leader(1, &make_pubkey(2)));
        assert!(!rotation.is_leader(1, &make_pubkey(1)));
    }

    #[test]
    fn test_empty_set() {
        let rotation = LeaderRotation::new(vec![]);
        assert_eq!(rotation.leader_for_view(0), None);
        assert!(!rotation.is_leader(0, &make_pubkey(1)));
    }

    #[test]
    fn test_single_validator() {
        let pk = make_pubkey(1);
        let rotation = LeaderRotation::new(vec![pk]);

        for view in 0..10 {
            assert_eq!(rotation.leader_for_view(view), Some(&pk));
            assert!(rotation.is_leader(view, &pk));
        }
    }
}
