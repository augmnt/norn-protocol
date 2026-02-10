//! Emergency pause/unpause pattern.
//!
//! Requires [`Ownable`](super::Ownable) to be initialized — only the owner
//! can pause or unpause the contract.
//!
//! ```ignore
//! use norn_sdk::prelude::*;
//!
//! fn init(ctx: &Context, _msg: Empty) -> Self {
//!     Ownable::init(&ctx.sender()).unwrap();
//!     Pausable::init().unwrap();
//!     MyContract
//! }
//!
//! fn execute(&mut self, ctx: &Context, msg: Exec) -> ContractResult {
//!     Pausable::require_not_paused()?;
//!     // ... logic ...
//! }
//! ```

use crate::contract::Context;
use crate::ensure;
use crate::error::ContractError;
use crate::response::{ContractResult, Event, Response};
use crate::stdlib::ownable::Ownable;
use crate::storage::Item;

const PAUSED_KEY: Item<bool> = Item::new("__pausable:paused");

/// Emergency pause/unpause control.
///
/// All methods are static — no instance needed. Depends on [`Ownable`] for
/// authorization. State is stored via the `__pausable:paused` storage key.
pub struct Pausable;

impl Pausable {
    /// Initialize the pausable state (unpaused). Call in your contract's `init()`.
    pub fn init() -> Result<(), ContractError> {
        PAUSED_KEY.save(&false)
    }

    /// Check if the contract is currently paused.
    pub fn is_paused() -> bool {
        PAUSED_KEY.load_or(false)
    }

    /// Assert that the contract is not paused.
    pub fn require_not_paused() -> Result<(), ContractError> {
        if Self::is_paused() {
            return Err(ContractError::Custom(alloc::string::String::from(
                "contract is paused",
            )));
        }
        Ok(())
    }

    /// Pause the contract (owner-only).
    pub fn pause(ctx: &Context) -> ContractResult {
        Ownable::require_owner(ctx)?;
        ensure!(!Self::is_paused(), "contract is already paused");
        PAUSED_KEY.save(&true)?;
        Ok(Response::new().add_event(Event::new("Paused")))
    }

    /// Unpause the contract (owner-only).
    pub fn unpause(ctx: &Context) -> ContractResult {
        Ownable::require_owner(ctx)?;
        ensure!(Self::is_paused(), "contract is not paused");
        PAUSED_KEY.save(&false)?;
        Ok(Response::new().add_event(Event::new("Unpaused")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    fn setup() -> TestEnv {
        let env = TestEnv::new().with_sender(ALICE);
        Ownable::init(&ALICE).unwrap();
        Pausable::init().unwrap();
        env
    }

    #[test]
    fn test_init_not_paused() {
        let _env = setup();
        assert!(!Pausable::is_paused());
        assert!(Pausable::require_not_paused().is_ok());
    }

    #[test]
    fn test_pause() {
        let env = setup();
        let resp = Pausable::pause(&env.ctx()).unwrap();
        assert!(Pausable::is_paused());
        assert_event(&resp, "Paused");
    }

    #[test]
    fn test_pause_then_require_fails() {
        let env = setup();
        Pausable::pause(&env.ctx()).unwrap();
        let err = Pausable::require_not_paused().unwrap_err();
        assert_eq!(err.message(), "contract is paused");
    }

    #[test]
    fn test_unpause() {
        let env = setup();
        Pausable::pause(&env.ctx()).unwrap();
        let resp = Pausable::unpause(&env.ctx()).unwrap();
        assert!(!Pausable::is_paused());
        assert_event(&resp, "Unpaused");
    }

    #[test]
    fn test_pause_unauthorized() {
        let env = setup();
        env.set_sender(BOB);
        let err = Pausable::pause(&env.ctx()).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized);
    }

    #[test]
    fn test_pause_already_paused() {
        let env = setup();
        Pausable::pause(&env.ctx()).unwrap();
        let err = Pausable::pause(&env.ctx()).unwrap_err();
        assert_eq!(err.message(), "contract is already paused");
    }

    #[test]
    fn test_unpause_not_paused() {
        let env = setup();
        let err = Pausable::unpause(&env.ctx()).unwrap_err();
        assert_eq!(err.message(), "contract is not paused");
    }
}
