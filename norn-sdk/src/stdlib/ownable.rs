//! Single-owner access control pattern.
//!
//! ```ignore
//! use norn_sdk::prelude::*;
//!
//! fn init(ctx: &Context, _msg: Empty) -> Self {
//!     Ownable::init(&ctx.sender()).unwrap();
//!     MyContract
//! }
//!
//! fn execute(&mut self, ctx: &Context, msg: Exec) -> ContractResult {
//!     Ownable::require_owner(ctx)?;
//!     // ... owner-only logic ...
//! }
//! ```

use crate::addr::{addr_to_hex, ZERO_ADDRESS};
use crate::contract::Context;
use crate::ensure_ne;
use crate::error::ContractError;
use crate::response::{ContractResult, Event, Response};
use crate::storage::Item;
use crate::types::Address;

const OWNER_KEY: Item<Address> = Item::new("__ownable:owner");

/// Single-owner access control.
///
/// All methods are static — no instance needed. State is stored via
/// the `__ownable:owner` storage key.
pub struct Ownable;

impl Ownable {
    /// Set the initial owner. Call this in your contract's `init()`.
    pub fn init(owner: &Address) -> Result<(), ContractError> {
        OWNER_KEY.save(owner)
    }

    /// Get the current owner address.
    pub fn owner() -> Result<Address, ContractError> {
        OWNER_KEY.load()
    }

    /// Assert that the sender is the owner.
    pub fn require_owner(ctx: &Context) -> Result<(), ContractError> {
        let owner = OWNER_KEY.load()?;
        if ctx.sender() != owner {
            return Err(ContractError::Unauthorized);
        }
        Ok(())
    }

    /// Transfer ownership to a new address (owner-only).
    pub fn transfer_ownership(ctx: &Context, new_owner: &Address) -> ContractResult {
        Self::require_owner(ctx)?;
        ensure_ne!(*new_owner, ZERO_ADDRESS, "new owner cannot be zero address");
        let prev = OWNER_KEY.load()?;
        OWNER_KEY.save(new_owner)?;
        Ok(Response::new().add_event(
            Event::new("OwnershipTransferred")
                .add_attribute("previous_owner", addr_to_hex(&prev))
                .add_attribute("new_owner", addr_to_hex(new_owner)),
        ))
    }

    /// Renounce ownership, setting owner to the zero address (owner-only).
    ///
    /// **Warning**: This is irreversible. The contract will have no owner.
    pub fn renounce_ownership(ctx: &Context) -> ContractResult {
        Self::require_owner(ctx)?;
        let prev = OWNER_KEY.load()?;
        OWNER_KEY.save(&ZERO_ADDRESS)?;
        Ok(Response::new().add_event(
            Event::new("OwnershipTransferred")
                .add_attribute("previous_owner", addr_to_hex(&prev))
                .add_attribute("new_owner", addr_to_hex(&ZERO_ADDRESS)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    const ALICE: Address = [1u8; 20];
    const BOB: Address = [2u8; 20];

    #[test]
    fn test_init_and_owner() {
        let _env = TestEnv::new().with_sender(ALICE);
        Ownable::init(&ALICE).unwrap();
        assert_eq!(Ownable::owner().unwrap(), ALICE);
    }

    #[test]
    fn test_require_owner_pass() {
        let env = TestEnv::new().with_sender(ALICE);
        Ownable::init(&ALICE).unwrap();
        assert!(Ownable::require_owner(&env.ctx()).is_ok());
    }

    #[test]
    fn test_require_owner_fail() {
        let env = TestEnv::new().with_sender(BOB);
        Ownable::init(&ALICE).unwrap();
        let err = Ownable::require_owner(&env.ctx()).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_transfer_ownership() {
        let env = TestEnv::new().with_sender(ALICE);
        Ownable::init(&ALICE).unwrap();
        let resp = Ownable::transfer_ownership(&env.ctx(), &BOB).unwrap();
        assert_eq!(Ownable::owner().unwrap(), BOB);
        assert_event(&resp, "OwnershipTransferred");
    }

    #[test]
    fn test_transfer_ownership_unauthorized() {
        let env = TestEnv::new().with_sender(BOB);
        Ownable::init(&ALICE).unwrap();
        let err = Ownable::transfer_ownership(&env.ctx(), &BOB).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_transfer_to_zero_fails() {
        let env = TestEnv::new().with_sender(ALICE);
        Ownable::init(&ALICE).unwrap();
        let err = Ownable::transfer_ownership(&env.ctx(), &ZERO_ADDRESS).unwrap_err();
        assert_eq!(err.message(), "new owner cannot be zero address");
    }

    #[test]
    fn test_renounce_ownership() {
        let env = TestEnv::new().with_sender(ALICE);
        Ownable::init(&ALICE).unwrap();
        let resp = Ownable::renounce_ownership(&env.ctx()).unwrap();
        assert_eq!(Ownable::owner().unwrap(), ZERO_ADDRESS);
        assert_event(&resp, "OwnershipTransferred");
    }
}
