use norn_crypto::keys::Keypair;
use norn_types::primitives::{Address, Amount};

use crate::error::NodeError;

/// A simple testnet faucet that can dispense tokens to addresses.
pub struct Faucet {
    keypair: Keypair,
    amount_per_request: Amount,
}

impl Faucet {
    /// Create a new faucet with the given keypair and dispense amount.
    pub fn new(keypair: Keypair, amount_per_request: Amount) -> Self {
        Self {
            keypair,
            amount_per_request,
        }
    }

    /// Dispense tokens to the given address.
    ///
    /// In a full implementation, this would create and submit a transfer knot
    /// to the network. For now, it serves as a structural placeholder.
    pub fn dispense(&self, address: Address) -> Result<(), NodeError> {
        tracing::info!(
            to = %hex::encode(address),
            amount = self.amount_per_request,
            from = %hex::encode(norn_crypto::address::pubkey_to_address(&self.keypair.public_key())),
            "faucet dispense requested"
        );

        // Placeholder: actual implementation would create a knot and
        // broadcast it to the network.
        Ok(())
    }

    /// Get the amount dispensed per request.
    pub fn amount_per_request(&self) -> Amount {
        self.amount_per_request
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::constants::ONE_NORN;

    #[test]
    fn test_faucet_creation() {
        let keypair = Keypair::generate();
        let faucet = Faucet::new(keypair, 10 * ONE_NORN);
        assert_eq!(faucet.amount_per_request(), 10 * ONE_NORN);
    }

    #[test]
    fn test_faucet_dispense() {
        let keypair = Keypair::generate();
        let faucet = Faucet::new(keypair, ONE_NORN);
        let addr = [42u8; 20];
        let result = faucet.dispense(addr);
        assert!(result.is_ok());
    }
}
