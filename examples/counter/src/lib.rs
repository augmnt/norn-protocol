//! Counter contract — demonstrates `#[norn_contract]` proc macro with zero
//! ceremony: just struct fields and annotated methods.

#![no_std]

extern crate alloc;

use norn_sdk::prelude::*;

#[norn_contract]
pub struct Counter {
    value: u64,
}

#[norn_contract]
impl Counter {
    #[init]
    pub fn new(_ctx: &Context) -> Self {
        Counter { value: 0 }
    }

    #[execute]
    pub fn increment(&mut self, _ctx: &Context) -> ContractResult {
        self.value += 1;
        Ok(Response::new()
            .add_attribute("action", "increment")
            .set_data(&self.value))
    }

    #[execute]
    pub fn decrement(&mut self, _ctx: &Context) -> ContractResult {
        ensure!(self.value > 0, "counter is already zero");
        self.value -= 1;
        Ok(Response::new()
            .add_attribute("action", "decrement")
            .set_data(&self.value))
    }

    #[execute]
    pub fn reset(&mut self, _ctx: &Context) -> ContractResult {
        self.value = 0;
        Ok(Response::new()
            .add_attribute("action", "reset")
            .set_data(&self.value))
    }

    #[query]
    pub fn get_value(&self, _ctx: &Context) -> ContractResult {
        ok(self.value)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    #[test]
    fn test_init() {
        let env = TestEnv::new();
        let counter = Counter::new(&env.ctx());
        assert_eq!(counter.value, 0);
    }

    #[test]
    fn test_increment() {
        let env = TestEnv::new();
        let mut counter = Counter::new(&env.ctx());
        let resp = counter.increment(&env.ctx()).unwrap();
        assert_attribute(&resp, "action", "increment");
        let val: u64 = from_response(&resp).unwrap();
        assert_eq!(val, 1);
    }

    #[test]
    fn test_decrement() {
        let env = TestEnv::new();
        let mut counter = Counter::new(&env.ctx());
        counter.increment(&env.ctx()).unwrap();
        let resp = counter.decrement(&env.ctx()).unwrap();
        assert_attribute(&resp, "action", "decrement");
        let val: u64 = from_response(&resp).unwrap();
        assert_eq!(val, 0);
    }

    #[test]
    fn test_decrement_at_zero_fails() {
        let env = TestEnv::new();
        let mut counter = Counter::new(&env.ctx());
        let err = counter.decrement(&env.ctx()).unwrap_err();
        assert_eq!(err.message(), "counter is already zero");
    }

    #[test]
    fn test_reset() {
        let env = TestEnv::new();
        let mut counter = Counter::new(&env.ctx());
        counter.increment(&env.ctx()).unwrap();
        counter.increment(&env.ctx()).unwrap();
        let resp = counter.reset(&env.ctx()).unwrap();
        let val: u64 = from_response(&resp).unwrap();
        assert_eq!(val, 0);
    }

    #[test]
    fn test_query() {
        let env = TestEnv::new();
        let mut counter = Counter::new(&env.ctx());
        counter.increment(&env.ctx()).unwrap();
        let resp = counter.get_value(&env.ctx()).unwrap();
        let val: u64 = from_response(&resp).unwrap();
        assert_eq!(val, 1);
    }
}
