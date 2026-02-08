use borsh::BorshSerialize;

use norn_crypto::hash::blake3_hash;
use norn_crypto::keys::{batch_verify, Keypair};
use norn_crypto::merkle::SparseMerkleTree;
use norn_types::constants::MAX_COMMITMENTS_PER_BLOCK;
use norn_types::primitives::*;
use norn_types::weave::{ValidatorSet, ValidatorSignature, WeaveBlock};

use crate::error::WeaveError;
use crate::mempool::BlockContents;

/// Build a weave block from the given contents.
///
/// Computes Merkle roots for each content category, produces the block hash,
/// and signs the block with the proposer's keypair.
pub fn build_block(
    prev_hash: Hash,
    prev_height: u64,
    contents: BlockContents,
    proposer_keypair: &Keypair,
    timestamp: Timestamp,
) -> WeaveBlock {
    let commitments_root = compute_merkle_root_borsh(&contents.commitments);
    let registrations_root = compute_merkle_root_borsh(&contents.registrations);
    let anchors_root = compute_merkle_root_borsh(&contents.anchors);
    let name_registrations_root = compute_merkle_root_borsh(&contents.name_registrations);
    let fraud_proofs_root = compute_merkle_root_borsh(&contents.fraud_proofs);
    let transfers_root = compute_merkle_root_borsh(&contents.transfers);
    let token_definitions_root = compute_merkle_root_borsh(&contents.token_definitions);
    let token_mints_root = compute_merkle_root_borsh(&contents.token_mints);
    let token_burns_root = compute_merkle_root_borsh(&contents.token_burns);
    let loom_deploys_root = compute_merkle_root_borsh(&contents.loom_deploys);

    let mut block = WeaveBlock {
        height: prev_height + 1,
        hash: [0u8; 32],
        prev_hash,
        commitments_root,
        registrations_root,
        anchors_root,
        commitments: contents.commitments,
        registrations: contents.registrations,
        anchors: contents.anchors,
        name_registrations: contents.name_registrations,
        name_registrations_root,
        fraud_proofs: contents.fraud_proofs,
        fraud_proofs_root,
        transfers: contents.transfers,
        transfers_root,
        token_definitions: contents.token_definitions,
        token_definitions_root,
        token_mints: contents.token_mints,
        token_mints_root,
        token_burns: contents.token_burns,
        token_burns_root,
        loom_deploys: contents.loom_deploys,
        loom_deploys_root,
        timestamp,
        proposer: proposer_keypair.public_key(),
        validator_signatures: Vec::new(),
    };

    block.hash = compute_block_hash(&block);

    // The proposer signs the block hash.
    let sig = proposer_keypair.sign(&block.hash);
    block.validator_signatures.push(ValidatorSignature {
        validator: proposer_keypair.public_key(),
        signature: sig,
    });

    block
}

/// Compute a deterministic block hash from all fields except the hash itself
/// and the validator signatures.
pub fn compute_block_hash(block: &WeaveBlock) -> Hash {
    let mut data = Vec::new();
    data.extend_from_slice(&block.height.to_le_bytes());
    data.extend_from_slice(&block.prev_hash);
    data.extend_from_slice(&block.commitments_root);
    data.extend_from_slice(&block.registrations_root);
    data.extend_from_slice(&block.anchors_root);
    data.extend_from_slice(&block.name_registrations_root);
    data.extend_from_slice(&block.fraud_proofs_root);
    data.extend_from_slice(&block.transfers_root);
    data.extend_from_slice(&block.token_definitions_root);
    data.extend_from_slice(&block.token_mints_root);
    data.extend_from_slice(&block.token_burns_root);
    data.extend_from_slice(&block.loom_deploys_root);
    data.extend_from_slice(&block.timestamp.to_le_bytes());
    data.extend_from_slice(&block.proposer);

    // Include content hashes for determinism.
    if let Ok(c_bytes) = borsh::to_vec(&block.commitments) {
        data.extend_from_slice(&blake3_hash(&c_bytes));
    }
    if let Ok(r_bytes) = borsh::to_vec(&block.registrations) {
        data.extend_from_slice(&blake3_hash(&r_bytes));
    }
    if let Ok(a_bytes) = borsh::to_vec(&block.anchors) {
        data.extend_from_slice(&blake3_hash(&a_bytes));
    }
    if let Ok(nr_bytes) = borsh::to_vec(&block.name_registrations) {
        data.extend_from_slice(&blake3_hash(&nr_bytes));
    }
    if let Ok(f_bytes) = borsh::to_vec(&block.fraud_proofs) {
        data.extend_from_slice(&blake3_hash(&f_bytes));
    }
    if let Ok(t_bytes) = borsh::to_vec(&block.transfers) {
        data.extend_from_slice(&blake3_hash(&t_bytes));
    }
    if let Ok(td_bytes) = borsh::to_vec(&block.token_definitions) {
        data.extend_from_slice(&blake3_hash(&td_bytes));
    }
    if let Ok(tm_bytes) = borsh::to_vec(&block.token_mints) {
        data.extend_from_slice(&blake3_hash(&tm_bytes));
    }
    if let Ok(tb_bytes) = borsh::to_vec(&block.token_burns) {
        data.extend_from_slice(&blake3_hash(&tb_bytes));
    }
    if let Ok(ld_bytes) = borsh::to_vec(&block.loom_deploys) {
        data.extend_from_slice(&blake3_hash(&ld_bytes));
    }

    blake3_hash(&data)
}

/// Verify a block's hash, proposer membership, Merkle roots, and validator signatures.
pub fn verify_block(block: &WeaveBlock, validator_set: &ValidatorSet) -> Result<(), WeaveError> {
    // 0. Reject oversized blocks.
    if block.commitments.len() > MAX_COMMITMENTS_PER_BLOCK {
        return Err(WeaveError::InvalidBlock {
            reason: format!(
                "too many commitments: {} > {}",
                block.commitments.len(),
                MAX_COMMITMENTS_PER_BLOCK
            ),
        });
    }

    // 1. Verify block hash matches recomputed hash.
    let expected_hash = compute_block_hash(block);
    if block.hash != expected_hash {
        return Err(WeaveError::InvalidBlock {
            reason: "block hash mismatch".to_string(),
        });
    }

    // 2. Verify proposer is in the validator set.
    if !validator_set.contains(&block.proposer) {
        return Err(WeaveError::InvalidBlock {
            reason: "proposer not in validator set".to_string(),
        });
    }

    // 3. Verify Merkle roots.
    let expected_commitments_root = compute_merkle_root_borsh(&block.commitments);
    if block.commitments_root != expected_commitments_root {
        return Err(WeaveError::InvalidBlock {
            reason: "commitments merkle root mismatch".to_string(),
        });
    }

    let expected_registrations_root = compute_merkle_root_borsh(&block.registrations);
    if block.registrations_root != expected_registrations_root {
        return Err(WeaveError::InvalidBlock {
            reason: "registrations merkle root mismatch".to_string(),
        });
    }

    let expected_anchors_root = compute_merkle_root_borsh(&block.anchors);
    if block.anchors_root != expected_anchors_root {
        return Err(WeaveError::InvalidBlock {
            reason: "anchors merkle root mismatch".to_string(),
        });
    }

    let expected_name_registrations_root = compute_merkle_root_borsh(&block.name_registrations);
    if block.name_registrations_root != expected_name_registrations_root {
        return Err(WeaveError::InvalidBlock {
            reason: "name registrations merkle root mismatch".to_string(),
        });
    }

    let expected_fraud_proofs_root = compute_merkle_root_borsh(&block.fraud_proofs);
    if block.fraud_proofs_root != expected_fraud_proofs_root {
        return Err(WeaveError::InvalidBlock {
            reason: "fraud proofs merkle root mismatch".to_string(),
        });
    }

    let expected_transfers_root = compute_merkle_root_borsh(&block.transfers);
    if block.transfers_root != expected_transfers_root {
        return Err(WeaveError::InvalidBlock {
            reason: "transfers merkle root mismatch".to_string(),
        });
    }

    let expected_token_definitions_root = compute_merkle_root_borsh(&block.token_definitions);
    if block.token_definitions_root != expected_token_definitions_root {
        return Err(WeaveError::InvalidBlock {
            reason: "token definitions merkle root mismatch".to_string(),
        });
    }

    let expected_token_mints_root = compute_merkle_root_borsh(&block.token_mints);
    if block.token_mints_root != expected_token_mints_root {
        return Err(WeaveError::InvalidBlock {
            reason: "token mints merkle root mismatch".to_string(),
        });
    }

    let expected_token_burns_root = compute_merkle_root_borsh(&block.token_burns);
    if block.token_burns_root != expected_token_burns_root {
        return Err(WeaveError::InvalidBlock {
            reason: "token burns merkle root mismatch".to_string(),
        });
    }

    let expected_loom_deploys_root = compute_merkle_root_borsh(&block.loom_deploys);
    if block.loom_deploys_root != expected_loom_deploys_root {
        return Err(WeaveError::InvalidBlock {
            reason: "loom deploys merkle root mismatch".to_string(),
        });
    }

    // 4. Verify validator signatures (need at least quorum_size) using batch verification.
    let quorum = validator_set.quorum_size();

    // Filter to only signatures from known validators.
    let valid_entries: Vec<_> = block
        .validator_signatures
        .iter()
        .filter(|vs| validator_set.contains(&vs.validator))
        .collect();

    if valid_entries.len() < quorum {
        return Err(WeaveError::InsufficientQuorum {
            have: valid_entries.len(),
            need: quorum,
        });
    }

    // Use batch verification for all validator signatures at once.
    let messages: Vec<&[u8]> = valid_entries
        .iter()
        .map(|_| block.hash.as_slice())
        .collect();
    let signatures: Vec<_> = valid_entries.iter().map(|vs| vs.signature).collect();
    let pubkeys: Vec<_> = valid_entries.iter().map(|vs| vs.validator).collect();

    batch_verify(&messages, &signatures, &pubkeys).map_err(|_| WeaveError::InsufficientQuorum {
        have: 0,
        need: quorum,
    })?;

    Ok(())
}

/// Compute a Merkle root for a list of borsh-serializable items.
/// Each item is keyed by the blake3 hash of its borsh-serialized form.
fn compute_merkle_root_borsh<T: BorshSerialize>(items: &[T]) -> Hash {
    let mut tree = SparseMerkleTree::new();
    for item in items {
        if let Ok(bytes) = borsh::to_vec(item) {
            let key = blake3_hash(&bytes);
            tree.insert(key, bytes);
        }
    }
    tree.root()
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::weave::{Validator, ValidatorSet};

    fn make_validator_set(keypairs: &[&Keypair]) -> ValidatorSet {
        let validators: Vec<Validator> = keypairs
            .iter()
            .map(|kp| Validator {
                pubkey: kp.public_key(),
                address: [0u8; 20],
                stake: 1000,
                active: true,
            })
            .collect();
        let total_stake = validators.len() as Amount * 1000;
        ValidatorSet {
            validators,
            total_stake,
            epoch: 0,
        }
    }

    #[test]
    fn test_build_and_verify_block() {
        let kp = Keypair::generate();
        let contents = crate::mempool::BlockContents {
            commitments: vec![],
            registrations: vec![],
            anchors: vec![],
            name_registrations: vec![],
            fraud_proofs: vec![],
            transfers: vec![],
            token_definitions: vec![],
            token_mints: vec![],
            token_burns: vec![],
            loom_deploys: vec![],
        };

        let block = build_block([0u8; 32], 0, contents, &kp, 1000);

        assert_eq!(block.height, 1);
        assert_ne!(block.hash, [0u8; 32]);
        assert_eq!(block.proposer, kp.public_key());
        assert_eq!(block.validator_signatures.len(), 1);

        let vs = make_validator_set(&[&kp]);
        assert!(verify_block(&block, &vs).is_ok());
    }

    #[test]
    fn test_block_hash_deterministic() {
        let kp = Keypair::generate();
        let contents = crate::mempool::BlockContents {
            commitments: vec![],
            registrations: vec![],
            anchors: vec![],
            name_registrations: vec![],
            fraud_proofs: vec![],
            transfers: vec![],
            token_definitions: vec![],
            token_mints: vec![],
            token_burns: vec![],
            loom_deploys: vec![],
        };
        let block = build_block([0u8; 32], 0, contents, &kp, 1000);

        let hash1 = compute_block_hash(&block);
        let hash2 = compute_block_hash(&block);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_verify_rejects_tampered_hash() {
        let kp = Keypair::generate();
        let contents = crate::mempool::BlockContents {
            commitments: vec![],
            registrations: vec![],
            anchors: vec![],
            name_registrations: vec![],
            fraud_proofs: vec![],
            transfers: vec![],
            token_definitions: vec![],
            token_mints: vec![],
            token_burns: vec![],
            loom_deploys: vec![],
        };
        let mut block = build_block([0u8; 32], 0, contents, &kp, 1000);
        block.hash[0] ^= 0xff;

        let vs = make_validator_set(&[&kp]);
        assert!(verify_block(&block, &vs).is_err());
    }

    #[test]
    fn test_verify_rejects_non_validator_proposer() {
        let kp = Keypair::generate();
        let other_kp = Keypair::generate();
        let contents = crate::mempool::BlockContents {
            commitments: vec![],
            registrations: vec![],
            anchors: vec![],
            name_registrations: vec![],
            fraud_proofs: vec![],
            transfers: vec![],
            token_definitions: vec![],
            token_mints: vec![],
            token_burns: vec![],
            loom_deploys: vec![],
        };
        let block = build_block([0u8; 32], 0, contents, &kp, 1000);

        // Validator set only has other_kp.
        let vs = make_validator_set(&[&other_kp]);
        assert!(verify_block(&block, &vs).is_err());
    }

    #[test]
    fn test_merkle_roots_match_contents() {
        use norn_types::weave::CommitmentUpdate;
        let kp = Keypair::generate();
        let commitment = CommitmentUpdate {
            thread_id: [1u8; 20],
            owner: [2u8; 32],
            version: 1,
            state_hash: [3u8; 32],
            prev_commitment_hash: [0u8; 32],
            knot_count: 1,
            timestamp: 1000,
            signature: [0u8; 64],
        };
        let contents = crate::mempool::BlockContents {
            commitments: vec![commitment],
            registrations: vec![],
            anchors: vec![],
            name_registrations: vec![],
            fraud_proofs: vec![],
            transfers: vec![],
            token_definitions: vec![],
            token_mints: vec![],
            token_burns: vec![],
            loom_deploys: vec![],
        };
        let block = build_block([0u8; 32], 0, contents, &kp, 1000);

        // The commitments root should not be the empty hash.
        assert_ne!(block.commitments_root, [0u8; 32]);
    }

    #[test]
    fn test_verify_rejects_oversized_block() {
        use norn_types::weave::CommitmentUpdate;
        let kp = Keypair::generate();
        let contents = crate::mempool::BlockContents {
            commitments: vec![],
            registrations: vec![],
            anchors: vec![],
            name_registrations: vec![],
            fraud_proofs: vec![],
            transfers: vec![],
            token_definitions: vec![],
            token_mints: vec![],
            token_burns: vec![],
            loom_deploys: vec![],
        };
        let mut block = build_block([0u8; 32], 0, contents, &kp, 1000);
        let vs = make_validator_set(&[&kp]);

        // Inject more commitments than allowed directly into the block.
        block.commitments = (0..MAX_COMMITMENTS_PER_BLOCK + 1)
            .map(|i| CommitmentUpdate {
                thread_id: [0u8; 20],
                owner: [0u8; 32],
                version: i as u64,
                state_hash: [0u8; 32],
                prev_commitment_hash: [0u8; 32],
                knot_count: 0,
                timestamp: 0,
                signature: [0u8; 64],
            })
            .collect();

        let result = verify_block(&block, &vs);
        assert!(result.is_err());
    }
}
