//! Counter contract — demonstrates SDK v3 features: Response builder, ensure!,
//! and native testing with TestEnv.
//!
//! Actions: Increment, Decrement, Reset.
//! Query: GetValue returns the current counter as u64.

#![no_std]

extern crate alloc;

use norn_sdk::prelude::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Counter {
    value: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Execute {
    Increment,
    Decrement,
    Reset,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Query {
    GetValue,
}

impl Contract for Counter {
    type Exec = Execute;
    type Query = Query;

    fn init(_ctx: &Context) -> Self {
        Counter { value: 0 }
    }

    fn execute(&mut self, _ctx: &Context, msg: Execute) -> ContractResult {
        match msg {
            Execute::Increment => {
                self.value += 1;
                Ok(Response::new()
                    .add_attribute("action", "increment")
                    .set_data(&self.value))
            }
            Execute::Decrement => {
                ensure!(self.value > 0, "counter is already zero");
                self.value -= 1;
                Ok(Response::new()
                    .add_attribute("action", "decrement")
                    .set_data(&self.value))
            }
            Execute::Reset => {
                self.value = 0;
                Ok(Response::new()
                    .add_attribute("action", "reset")
                    .set_data(&self.value))
            }
        }
    }

    fn query(&self, _ctx: &Context, msg: Query) -> ContractResult {
        match msg {
            Query::GetValue => ok(self.value),
        }
    }
}

norn_entry!(Counter);

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    #[test]
    fn test_init() {
        let env = TestEnv::new();
        let counter = Counter::init(&env.ctx());
        assert_eq!(counter.value, 0);
    }

    #[test]
    fn test_increment() {
        let env = TestEnv::new();
        let mut counter = Counter::init(&env.ctx());
        let resp = counter.execute(&env.ctx(), Execute::Increment).unwrap();
        assert_attribute(&resp, "action", "increment");
        let val: u64 = from_response(&resp).unwrap();
        assert_eq!(val, 1);
    }

    #[test]
    fn test_decrement() {
        let env = TestEnv::new();
        let mut counter = Counter::init(&env.ctx());
        counter.execute(&env.ctx(), Execute::Increment).unwrap();
        let resp = counter.execute(&env.ctx(), Execute::Decrement).unwrap();
        assert_attribute(&resp, "action", "decrement");
        let val: u64 = from_response(&resp).unwrap();
        assert_eq!(val, 0);
    }

    #[test]
    fn test_decrement_at_zero_fails() {
        let env = TestEnv::new();
        let mut counter = Counter::init(&env.ctx());
        let err = counter.execute(&env.ctx(), Execute::Decrement).unwrap_err();
        assert_eq!(err.message(), "counter is already zero");
    }

    #[test]
    fn test_reset() {
        let env = TestEnv::new();
        let mut counter = Counter::init(&env.ctx());
        counter.execute(&env.ctx(), Execute::Increment).unwrap();
        counter.execute(&env.ctx(), Execute::Increment).unwrap();
        let resp = counter.execute(&env.ctx(), Execute::Reset).unwrap();
        let val: u64 = from_response(&resp).unwrap();
        assert_eq!(val, 0);
    }

    #[test]
    fn test_query() {
        let env = TestEnv::new();
        let mut counter = Counter::init(&env.ctx());
        counter.execute(&env.ctx(), Execute::Increment).unwrap();
        let resp = counter.query(&env.ctx(), Query::GetValue).unwrap();
        let val: u64 = from_response(&resp).unwrap();
        assert_eq!(val, 1);
    }
}
