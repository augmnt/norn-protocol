use std::collections::HashMap;

use norn_types::primitives::{Address, LoomId};

use crate::error::LoomError;
use crate::gas::MAX_CALL_DEPTH;

/// A single frame on the cross-contract call stack.
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// The loom (contract) being executed in this frame.
    pub loom_id: LoomId,
    /// The caller address for this frame.
    pub caller: Address,
    /// Snapshot of the loom's state before execution (for rollback on failure).
    pub state_snapshot: HashMap<Vec<u8>, Vec<u8>>,
    /// Gas used before this frame was pushed (for accounting).
    pub gas_before: u64,
}

/// Tracks the nested cross-contract call chain.
///
/// Enforces maximum call depth and prevents re-entrancy (a contract
/// cannot call itself, directly or transitively).
#[derive(Debug, Clone)]
pub struct CallStack {
    frames: Vec<CallFrame>,
    max_depth: u8,
}

impl CallStack {
    /// Create a new, empty call stack.
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            max_depth: MAX_CALL_DEPTH,
        }
    }

    /// Push a new frame onto the call stack.
    ///
    /// Returns an error if the maximum depth would be exceeded or if the
    /// target loom is already on the stack (re-entrancy).
    pub fn push(&mut self, frame: CallFrame) -> Result<(), LoomError> {
        let new_depth = self.frames.len() as u8 + 1;
        if new_depth > self.max_depth {
            return Err(LoomError::CallDepthExceeded {
                depth: new_depth,
                max: self.max_depth,
            });
        }
        if self.is_reentrant(&frame.loom_id) {
            return Err(LoomError::ReentrancyDetected {
                loom_id: frame.loom_id,
            });
        }
        self.frames.push(frame);
        Ok(())
    }

    /// Pop the most recent frame from the call stack.
    pub fn pop(&mut self) -> Option<CallFrame> {
        self.frames.pop()
    }

    /// Current call depth (number of frames on the stack).
    pub fn depth(&self) -> u8 {
        self.frames.len() as u8
    }

    /// Check if a loom is already on the call stack (re-entrancy check).
    pub fn is_reentrant(&self, loom_id: &LoomId) -> bool {
        self.frames.iter().any(|f| f.loom_id == *loom_id)
    }

    /// The caller address of the current (top) frame, if any.
    pub fn current_caller(&self) -> Option<Address> {
        self.frames.last().map(|f| f.caller)
    }

    /// The loom ID of the current (top) frame, if any.
    pub fn current_loom(&self) -> Option<LoomId> {
        self.frames.last().map(|f| f.loom_id)
    }

    /// Whether the stack is empty (top-level call).
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_frame(loom_id: LoomId, caller: Address) -> CallFrame {
        CallFrame {
            loom_id,
            caller,
            state_snapshot: HashMap::new(),
            gas_before: 0,
        }
    }

    #[test]
    fn test_push_and_pop() {
        let mut stack = CallStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.depth(), 0);

        stack.push(make_frame([1u8; 32], [1u8; 20])).unwrap();
        assert_eq!(stack.depth(), 1);
        assert!(!stack.is_empty());

        let frame = stack.pop().unwrap();
        assert_eq!(frame.loom_id, [1u8; 32]);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_depth_limit() {
        let mut stack = CallStack::new();
        for i in 0..MAX_CALL_DEPTH {
            let mut id = [0u8; 32];
            id[0] = i;
            stack.push(make_frame(id, [1u8; 20])).unwrap();
        }
        assert_eq!(stack.depth(), MAX_CALL_DEPTH);

        // One more should fail.
        let mut id = [0u8; 32];
        id[0] = MAX_CALL_DEPTH;
        let result = stack.push(make_frame(id, [1u8; 20]));
        assert!(result.is_err());
        match result.unwrap_err() {
            LoomError::CallDepthExceeded { depth, max } => {
                assert_eq!(depth, MAX_CALL_DEPTH + 1);
                assert_eq!(max, MAX_CALL_DEPTH);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn test_reentrancy_detection() {
        let mut stack = CallStack::new();
        let loom_a = [1u8; 32];
        let loom_b = [2u8; 32];

        stack.push(make_frame(loom_a, [1u8; 20])).unwrap();
        stack.push(make_frame(loom_b, [2u8; 20])).unwrap();

        // Trying to push loom_a again should fail (re-entrancy).
        let result = stack.push(make_frame(loom_a, [3u8; 20]));
        assert!(result.is_err());
        match result.unwrap_err() {
            LoomError::ReentrancyDetected { loom_id } => {
                assert_eq!(loom_id, loom_a);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn test_is_reentrant() {
        let mut stack = CallStack::new();
        let loom_a = [1u8; 32];
        let loom_b = [2u8; 32];

        assert!(!stack.is_reentrant(&loom_a));
        stack.push(make_frame(loom_a, [1u8; 20])).unwrap();
        assert!(stack.is_reentrant(&loom_a));
        assert!(!stack.is_reentrant(&loom_b));
    }

    #[test]
    fn test_current_caller() {
        let mut stack = CallStack::new();
        assert!(stack.current_caller().is_none());

        stack.push(make_frame([1u8; 32], [10u8; 20])).unwrap();
        assert_eq!(stack.current_caller(), Some([10u8; 20]));

        stack.push(make_frame([2u8; 32], [20u8; 20])).unwrap();
        assert_eq!(stack.current_caller(), Some([20u8; 20]));

        stack.pop();
        assert_eq!(stack.current_caller(), Some([10u8; 20]));
    }

    #[test]
    fn test_current_loom() {
        let mut stack = CallStack::new();
        assert!(stack.current_loom().is_none());

        stack.push(make_frame([1u8; 32], [10u8; 20])).unwrap();
        assert_eq!(stack.current_loom(), Some([1u8; 32]));
    }

    #[test]
    fn test_pop_empty() {
        let mut stack = CallStack::new();
        assert!(stack.pop().is_none());
    }
}
