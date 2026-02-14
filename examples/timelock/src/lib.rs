//! Time-locked Vault — deposit tokens with an unlock date.
//! Self-custody with a forced hold period.

#![no_std]

extern crate alloc;

use alloc::format;
use norn_sdk::prelude::*;

// ── Storage ────────────────────────────────────────────────────────────

const LOCK_COUNT: Item<u64> = Item::new("lock_count");
const LOCKS: Map<u64, LockInfo> = Map::new("locks");

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct LockInfo {
    pub id: u64,
    pub owner: Address,
    pub token_id: TokenId,
    pub amount: u128,
    pub unlock_time: u64,
    pub withdrawn: bool,
    pub created_at: u64,
}

// ── Contract ───────────────────────────────────────────────────────────

#[norn_contract]
pub struct Timelock;

#[norn_contract]
impl Timelock {
    #[init]
    pub fn new(_ctx: &Context) -> Self {
        LOCK_COUNT.init(&0u64);
        Timelock
    }

    #[execute]
    pub fn lock(
        &mut self,
        ctx: &Context,
        token_id: TokenId,
        amount: u128,
        unlock_time: u64,
    ) -> ContractResult {
        ensure!(amount > 0, "amount must be positive");
        ensure!(
            unlock_time > ctx.timestamp(),
            "unlock_time must be in the future"
        );

        let contract = ctx.contract_address();
        ctx.transfer(&ctx.sender(), &contract, &token_id, amount);

        let id = LOCK_COUNT.load_or(0u64);
        LOCKS.save(
            &id,
            &LockInfo {
                id,
                owner: ctx.sender(),
                token_id,
                amount,
                unlock_time,
                withdrawn: false,
                created_at: ctx.timestamp(),
            },
        )?;
        LOCK_COUNT.save(&safe_add_u64(id, 1)?)?;

        Ok(Response::with_action("lock")
            .add_attribute("lock_id", format!("{}", id))
            .set_data(&id))
    }

    #[execute]
    pub fn withdraw(&mut self, ctx: &Context, lock_id: u64) -> ContractResult {
        let mut lock = LOCKS.load(&lock_id)?;
        ensure!(ctx.sender() == lock.owner, "only owner can withdraw");
        ensure!(!lock.withdrawn, "already withdrawn");
        ensure!(
            ctx.timestamp() >= lock.unlock_time,
            "tokens are still locked"
        );

        ctx.transfer_from_contract(&lock.owner, &lock.token_id, lock.amount);
        lock.withdrawn = true;
        LOCKS.save(&lock_id, &lock)?;

        Ok(Response::with_action("withdraw")
            .add_attribute("lock_id", format!("{}", lock_id))
            .add_attribute("amount", format!("{}", lock.amount)))
    }

    #[query]
    pub fn get_lock(&self, _ctx: &Context, lock_id: u64) -> ContractResult {
        let lock = LOCKS.load(&lock_id)?;
        ok(lock)
    }

    #[query]
    pub fn get_lock_count(&self, _ctx: &Context) -> ContractResult {
        let count = LOCK_COUNT.load_or(0u64);
        ok(count)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    const TOKEN: TokenId = [42u8; 32];
    const CONTRACT_ADDR: Address = [99u8; 20];

    fn setup() -> (TestEnv, Timelock) {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let tl = Timelock::new(&env.ctx());
        (env, tl)
    }

    #[test]
    fn test_lock() {
        let (env, mut tl) = setup();
        let resp = tl.lock(&env.ctx(), TOKEN, 5_000, 2000).unwrap();
        let id: u64 = from_response(&resp).unwrap();
        assert_eq!(id, 0);

        let resp = tl.get_lock(&env.ctx(), 0).unwrap();
        let lock: LockInfo = from_response(&resp).unwrap();
        assert_eq!(lock.owner, ALICE);
        assert_eq!(lock.amount, 5_000);
        assert_eq!(lock.unlock_time, 2000);
        assert!(!lock.withdrawn);
    }

    #[test]
    fn test_withdraw_after_unlock() {
        let (env, mut tl) = setup();
        tl.lock(&env.ctx(), TOKEN, 5_000, 2000).unwrap();

        env.set_timestamp(2000);
        tl.withdraw(&env.ctx(), 0).unwrap();

        let resp = tl.get_lock(&env.ctx(), 0).unwrap();
        let lock: LockInfo = from_response(&resp).unwrap();
        assert!(lock.withdrawn);

        let transfers = env.transfers();
        assert_eq!(transfers.len(), 2); // lock + withdraw
        assert_eq!(transfers[1].1, ALICE.to_vec());
        assert_eq!(transfers[1].3, 5_000);
    }

    #[test]
    fn test_cannot_withdraw_before_unlock() {
        let (env, mut tl) = setup();
        tl.lock(&env.ctx(), TOKEN, 5_000, 2000).unwrap();

        env.set_timestamp(1500);
        let err = tl.withdraw(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "tokens are still locked");
    }

    #[test]
    fn test_cannot_withdraw_twice() {
        let (env, mut tl) = setup();
        tl.lock(&env.ctx(), TOKEN, 5_000, 2000).unwrap();

        env.set_timestamp(2000);
        tl.withdraw(&env.ctx(), 0).unwrap();
        let err = tl.withdraw(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "already withdrawn");
    }

    #[test]
    fn test_only_owner_can_withdraw() {
        let (env, mut tl) = setup();
        tl.lock(&env.ctx(), TOKEN, 5_000, 2000).unwrap();

        env.set_sender(BOB);
        env.set_timestamp(2000);
        let err = tl.withdraw(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "only owner can withdraw");
    }

    #[test]
    fn test_unlock_time_must_be_future() {
        let (env, mut tl) = setup();
        let err = tl.lock(&env.ctx(), TOKEN, 5_000, 500).unwrap_err();
        assert_err_contains(&err, "unlock_time must be in the future");
    }

    #[test]
    fn test_multiple_locks() {
        let (env, mut tl) = setup();
        tl.lock(&env.ctx(), TOKEN, 1_000, 2000).unwrap();
        tl.lock(&env.ctx(), TOKEN, 2_000, 3000).unwrap();

        let resp = tl.get_lock_count(&env.ctx()).unwrap();
        let count: u64 = from_response(&resp).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_lock_different_tokens() {
        let (env, mut tl) = setup();
        let token_b: TokenId = [55u8; 32];
        tl.lock(&env.ctx(), TOKEN, 1_000, 2000).unwrap();
        tl.lock(&env.ctx(), token_b, 2_000, 3000).unwrap();

        let resp = tl.get_lock(&env.ctx(), 1).unwrap();
        let lock: LockInfo = from_response(&resp).unwrap();
        assert_eq!(lock.token_id, token_b);
        assert_eq!(lock.amount, 2_000);
    }
}
