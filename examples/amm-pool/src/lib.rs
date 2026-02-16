//! AMM Pool — constant-product automated market maker with NORN base pairs.
//!
//! Every pool pairs a token with NORN. To swap Token A for Token B the path
//! is A -> NORN -> B (two hops). Liquidity providers earn swap fees (default
//! 0.3%) proportional to their share of the pool.

#![no_std]

extern crate alloc;

use alloc::format;
use norn_sdk::prelude::*;

// ── Storage ──────────────────────────────────────────────────────────────

const POOL_COUNT: Item<u64> = Item::new("pool_count");
const POOLS: Map<u64, Pool> = Map::new("pools");
const TOKEN_TO_POOL: Map<TokenId, u64> = Map::new("tok2pool");
const LP_BALANCES: Map<(u64, Address), u128> = Map::new("lp_bal");
const LP_TOTAL: Map<u64, u128> = Map::new("lp_tot");
const FEE_BPS: Item<u16> = Item::new("fee_bps");
const OWNER: Item<Address> = Item::new("owner");

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Pool {
    pub id: u64,
    pub token: TokenId,
    pub reserve_norn: u128,
    pub reserve_token: u128,
    pub created_at: u64,
}

// ── Math helpers ─────────────────────────────────────────────────────────

/// Integer square root via Newton's method (no floating point).
fn isqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Compute swap output using the constant-product formula with fee.
///
/// output = (amount_in_after_fee * reserve_out) / (reserve_in * 10000 + amount_in_after_fee)
fn compute_output(
    reserve_in: u128,
    reserve_out: u128,
    amount_in: u128,
    fee_bps: u16,
) -> Result<u128, ContractError> {
    let amount_in_after_fee = safe_mul(amount_in, 10000 - fee_bps as u128)?;
    let numerator = safe_mul(amount_in_after_fee, reserve_out)?;
    let denominator = safe_add(safe_mul(reserve_in, 10000)?, amount_in_after_fee)?;
    numerator
        .checked_div(denominator)
        .ok_or(ContractError::Overflow)
}

// ── Contract ─────────────────────────────────────────────────────────────

#[norn_contract]
pub struct AmmPool;

#[norn_contract]
impl AmmPool {
    #[init]
    pub fn new(ctx: &Context) -> Self {
        POOL_COUNT.init(&0u64);
        FEE_BPS.init(&30u16); // 0.3%
        OWNER.init(&ctx.sender());
        AmmPool
    }

    // ── Execute ──────────────────────────────────────────────────────

    /// Create a new liquidity pool pairing `token` with NORN.
    #[execute]
    pub fn create_pool(
        &mut self,
        ctx: &Context,
        token: TokenId,
        norn_amount: u128,
        token_amount: u128,
    ) -> ContractResult {
        ensure!(norn_amount > 0, "norn_amount must be positive");
        ensure!(token_amount > 0, "token_amount must be positive");
        ensure!(
            !TOKEN_TO_POOL.has(&token),
            "pool already exists for this token"
        );

        let contract = ctx.contract_address();
        let norn_token = [0u8; 32]; // NORN is the zero token

        // Transfer tokens to pool contract
        ctx.transfer(&ctx.sender(), &contract, &norn_token, norn_amount);
        ctx.transfer(&ctx.sender(), &contract, &token, token_amount);

        let id = POOL_COUNT.load_or(0u64);
        POOLS.save(
            &id,
            &Pool {
                id,
                token,
                reserve_norn: norn_amount,
                reserve_token: token_amount,
                created_at: ctx.timestamp(),
            },
        )?;
        TOKEN_TO_POOL.save(&token, &id)?;
        POOL_COUNT.save(&safe_add_u64(id, 1)?)?;

        // Mint initial LP tokens = sqrt(norn * token)
        let lp = isqrt(safe_mul(norn_amount, token_amount)?);
        ensure!(lp > 0, "insufficient initial liquidity");
        LP_BALANCES.save(&(id, ctx.sender()), &lp)?;
        LP_TOTAL.save(&id, &lp)?;

        Ok(Response::with_action("create_pool")
            .add_attribute("pool_id", format!("{}", id))
            .add_u128("lp_minted", lp)
            .set_data(&id))
    }

    /// Add proportional liquidity to an existing pool.
    #[execute]
    pub fn add_liquidity(
        &mut self,
        ctx: &Context,
        pool_id: u64,
        norn_amount: u128,
        token_amount: u128,
    ) -> ContractResult {
        ensure!(norn_amount > 0, "norn_amount must be positive");
        ensure!(token_amount > 0, "token_amount must be positive");

        let mut pool = POOLS.load(&pool_id)?;
        let total_lp = LP_TOTAL.load_or(&pool_id, 0u128);
        ensure!(total_lp > 0, "pool has no liquidity");

        let contract = ctx.contract_address();
        let norn_token = [0u8; 32];

        ctx.transfer(&ctx.sender(), &contract, &norn_token, norn_amount);
        ctx.transfer(&ctx.sender(), &contract, &pool.token, token_amount);

        // LP = min(norn * total_lp / reserve_norn, token * total_lp / reserve_token)
        let lp_norn = safe_mul(norn_amount, total_lp)?
            .checked_div(pool.reserve_norn)
            .ok_or(ContractError::Overflow)?;
        let lp_token = safe_mul(token_amount, total_lp)?
            .checked_div(pool.reserve_token)
            .ok_or(ContractError::Overflow)?;
        let lp = if lp_norn < lp_token {
            lp_norn
        } else {
            lp_token
        };
        ensure!(lp > 0, "insufficient liquidity amount");

        pool.reserve_norn = safe_add(pool.reserve_norn, norn_amount)?;
        pool.reserve_token = safe_add(pool.reserve_token, token_amount)?;
        POOLS.save(&pool_id, &pool)?;

        let prev = LP_BALANCES.load_or(&(pool_id, ctx.sender()), 0u128);
        LP_BALANCES.save(&(pool_id, ctx.sender()), &safe_add(prev, lp)?)?;
        LP_TOTAL.save(&pool_id, &safe_add(total_lp, lp)?)?;

        Ok(Response::with_action("add_liquidity")
            .add_attribute("pool_id", format!("{}", pool_id))
            .add_u128("lp_minted", lp))
    }

    /// Burn LP tokens and receive proportional NORN + token.
    #[execute]
    pub fn remove_liquidity(
        &mut self,
        ctx: &Context,
        pool_id: u64,
        lp_amount: u128,
    ) -> ContractResult {
        ensure!(lp_amount > 0, "lp_amount must be positive");

        let mut pool = POOLS.load(&pool_id)?;
        let total_lp = LP_TOTAL.load_or(&pool_id, 0u128);
        let user_lp = LP_BALANCES.load_or(&(pool_id, ctx.sender()), 0u128);
        ensure!(user_lp >= lp_amount, "insufficient LP balance");

        // Calculate share of reserves
        let norn_out = safe_mul(lp_amount, pool.reserve_norn)?
            .checked_div(total_lp)
            .ok_or(ContractError::Overflow)?;
        let token_out = safe_mul(lp_amount, pool.reserve_token)?
            .checked_div(total_lp)
            .ok_or(ContractError::Overflow)?;

        pool.reserve_norn = safe_sub(pool.reserve_norn, norn_out)?;
        pool.reserve_token = safe_sub(pool.reserve_token, token_out)?;
        POOLS.save(&pool_id, &pool)?;

        let new_lp = safe_sub(user_lp, lp_amount)?;
        LP_BALANCES.save(&(pool_id, ctx.sender()), &new_lp)?;
        LP_TOTAL.save(&pool_id, &safe_sub(total_lp, lp_amount)?)?;

        // Transfer tokens out
        let norn_token = [0u8; 32];
        ctx.transfer_from_contract(&ctx.sender(), &norn_token, norn_out);
        ctx.transfer_from_contract(&ctx.sender(), &pool.token, token_out);

        Ok(Response::with_action("remove_liquidity")
            .add_attribute("pool_id", format!("{}", pool_id))
            .add_u128("norn_out", norn_out)
            .add_u128("token_out", token_out))
    }

    /// Swap NORN for token with slippage protection.
    #[execute]
    pub fn swap_norn_for_token(
        &mut self,
        ctx: &Context,
        pool_id: u64,
        norn_amount: u128,
        min_token_out: u128,
    ) -> ContractResult {
        ensure!(norn_amount > 0, "norn_amount must be positive");

        let mut pool = POOLS.load(&pool_id)?;
        let fee_bps = FEE_BPS.load_or(30u16);

        let token_out =
            compute_output(pool.reserve_norn, pool.reserve_token, norn_amount, fee_bps)?;
        ensure!(token_out >= min_token_out, "slippage: output below minimum");
        ensure!(token_out > 0, "zero output");

        let contract = ctx.contract_address();
        let norn_token = [0u8; 32];
        ctx.transfer(&ctx.sender(), &contract, &norn_token, norn_amount);
        ctx.transfer_from_contract(&ctx.sender(), &pool.token, token_out);

        pool.reserve_norn = safe_add(pool.reserve_norn, norn_amount)?;
        pool.reserve_token = safe_sub(pool.reserve_token, token_out)?;
        POOLS.save(&pool_id, &pool)?;

        Ok(Response::with_action("swap_norn_for_token")
            .add_attribute("pool_id", format!("{}", pool_id))
            .add_u128("norn_in", norn_amount)
            .add_u128("token_out", token_out)
            .set_data(&token_out))
    }

    /// Swap token for NORN with slippage protection.
    #[execute]
    pub fn swap_token_for_norn(
        &mut self,
        ctx: &Context,
        pool_id: u64,
        token_amount: u128,
        min_norn_out: u128,
    ) -> ContractResult {
        ensure!(token_amount > 0, "token_amount must be positive");

        let mut pool = POOLS.load(&pool_id)?;
        let fee_bps = FEE_BPS.load_or(30u16);

        let norn_out =
            compute_output(pool.reserve_token, pool.reserve_norn, token_amount, fee_bps)?;
        ensure!(norn_out >= min_norn_out, "slippage: output below minimum");
        ensure!(norn_out > 0, "zero output");

        let contract = ctx.contract_address();
        let norn_token = [0u8; 32];
        ctx.transfer(&ctx.sender(), &contract, &pool.token, token_amount);
        ctx.transfer_from_contract(&ctx.sender(), &norn_token, norn_out);

        pool.reserve_token = safe_add(pool.reserve_token, token_amount)?;
        pool.reserve_norn = safe_sub(pool.reserve_norn, norn_out)?;
        POOLS.save(&pool_id, &pool)?;

        Ok(Response::with_action("swap_token_for_norn")
            .add_attribute("pool_id", format!("{}", pool_id))
            .add_u128("token_in", token_amount)
            .add_u128("norn_out", norn_out)
            .set_data(&norn_out))
    }

    /// Owner-only: update the swap fee (max 1000 = 10%).
    #[execute]
    pub fn set_fee_bps(&mut self, ctx: &Context, fee_bps: u16) -> ContractResult {
        let owner = OWNER.load()?;
        ensure!(ctx.sender() == owner, "only owner can set fee");
        ensure!(fee_bps <= 1000, "fee cannot exceed 10%");
        FEE_BPS.save(&fee_bps)?;

        Ok(Response::with_action("set_fee_bps").add_attribute("fee_bps", format!("{}", fee_bps)))
    }

    // ── Query ────────────────────────────────────────────────────────

    #[query]
    pub fn get_pool(&self, _ctx: &Context, pool_id: u64) -> ContractResult {
        let pool = POOLS.load(&pool_id)?;
        ok(pool)
    }

    #[query]
    pub fn get_pool_by_token(&self, _ctx: &Context, token: TokenId) -> ContractResult {
        let pool_id = TOKEN_TO_POOL.load(&token)?;
        let pool = POOLS.load(&pool_id)?;
        ok(pool)
    }

    #[query]
    pub fn get_pool_count(&self, _ctx: &Context) -> ContractResult {
        let count = POOL_COUNT.load_or(0u64);
        ok(count)
    }

    #[query]
    pub fn get_lp_balance(&self, _ctx: &Context, pool_id: u64, address: Address) -> ContractResult {
        let bal = LP_BALANCES.load_or(&(pool_id, address), 0u128);
        ok(bal)
    }

    #[query]
    pub fn get_quote(
        &self,
        _ctx: &Context,
        pool_id: u64,
        input_token_is_norn: bool,
        amount_in: u128,
    ) -> ContractResult {
        let pool = POOLS.load(&pool_id)?;
        let fee_bps = FEE_BPS.load_or(30u16);

        let output = if input_token_is_norn {
            compute_output(pool.reserve_norn, pool.reserve_token, amount_in, fee_bps)?
        } else {
            compute_output(pool.reserve_token, pool.reserve_norn, amount_in, fee_bps)?
        };
        ok(output)
    }

    #[query]
    pub fn get_config(&self, _ctx: &Context) -> ContractResult {
        let fee_bps = FEE_BPS.load_or(30u16);
        let owner = OWNER.load()?;
        ok((fee_bps, owner))
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    const TOKEN_A: TokenId = [1u8; 32];
    const TOKEN_B: TokenId = [2u8; 32];
    const CONTRACT_ADDR: Address = [99u8; 20];

    fn setup() -> (TestEnv, AmmPool) {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let pool = AmmPool::new(&env.ctx());
        (env, pool)
    }

    #[test]
    fn test_create_pool() {
        let (env, mut amm) = setup();
        let resp = amm
            .create_pool(&env.ctx(), TOKEN_A, 10_000, 20_000)
            .unwrap();

        let id: u64 = from_response(&resp).unwrap();
        assert_eq!(id, 0);

        let resp = amm.get_pool(&env.ctx(), 0).unwrap();
        let pool: Pool = from_response(&resp).unwrap();
        assert_eq!(pool.reserve_norn, 10_000);
        assert_eq!(pool.reserve_token, 20_000);
        assert_eq!(pool.token, TOKEN_A);

        // LP = isqrt(10000 * 20000) = isqrt(200_000_000) = 14142
        let resp = amm.get_lp_balance(&env.ctx(), 0, ALICE).unwrap();
        let lp: u128 = from_response(&resp).unwrap();
        assert_eq!(lp, isqrt(10_000 * 20_000));

        // Transfers: NORN to contract, TOKEN_A to contract
        assert_eq!(env.transfers().len(), 2);
    }

    #[test]
    fn test_add_liquidity() {
        let (env, mut amm) = setup();
        amm.create_pool(&env.ctx(), TOKEN_A, 10_000, 20_000)
            .unwrap();

        let initial_lp = isqrt(10_000 * 20_000);

        // Add proportional liquidity (same ratio)
        let resp = amm.add_liquidity(&env.ctx(), 0, 5_000, 10_000).unwrap();
        assert_attribute(&resp, "action", "add_liquidity");

        let resp = amm.get_pool(&env.ctx(), 0).unwrap();
        let pool: Pool = from_response(&resp).unwrap();
        assert_eq!(pool.reserve_norn, 15_000);
        assert_eq!(pool.reserve_token, 30_000);

        let resp = amm.get_lp_balance(&env.ctx(), 0, ALICE).unwrap();
        let lp: u128 = from_response(&resp).unwrap();
        // LP minted = min(5000 * initial_lp / 10000, 10000 * initial_lp / 20000)
        // = initial_lp / 2
        let expected_new = initial_lp / 2;
        assert_eq!(lp, initial_lp + expected_new);
    }

    #[test]
    fn test_remove_liquidity() {
        let (env, mut amm) = setup();
        amm.create_pool(&env.ctx(), TOKEN_A, 10_000, 20_000)
            .unwrap();

        let resp = amm.get_lp_balance(&env.ctx(), 0, ALICE).unwrap();
        let total_lp: u128 = from_response(&resp).unwrap();

        // Remove half
        let half = total_lp / 2;
        let resp = amm.remove_liquidity(&env.ctx(), 0, half).unwrap();
        assert_attribute(&resp, "action", "remove_liquidity");

        let resp = amm.get_pool(&env.ctx(), 0).unwrap();
        let pool: Pool = from_response(&resp).unwrap();
        // Reserves should be approximately halved
        assert_eq!(pool.reserve_norn, 10_000 - 10_000 * half / total_lp);
        assert_eq!(pool.reserve_token, 20_000 - 20_000 * half / total_lp);

        let resp = amm.get_lp_balance(&env.ctx(), 0, ALICE).unwrap();
        let remaining: u128 = from_response(&resp).unwrap();
        assert_eq!(remaining, total_lp - half);
    }

    #[test]
    fn test_swap_norn_for_token() {
        let (env, mut amm) = setup();
        amm.create_pool(&env.ctx(), TOKEN_A, 100_000, 200_000)
            .unwrap();

        let k_before = 100_000u128 * 200_000u128;

        env.set_sender(BOB);
        let resp = amm.swap_norn_for_token(&env.ctx(), 0, 1_000, 0).unwrap();
        let token_out: u128 = from_response(&resp).unwrap();
        assert!(token_out > 0);

        let resp = amm.get_pool(&env.ctx(), 0).unwrap();
        let pool: Pool = from_response(&resp).unwrap();
        // k should not decrease (increases slightly due to fees)
        let k_after = pool.reserve_norn * pool.reserve_token;
        assert!(k_after >= k_before);
    }

    #[test]
    fn test_swap_token_for_norn() {
        let (env, mut amm) = setup();
        amm.create_pool(&env.ctx(), TOKEN_A, 100_000, 200_000)
            .unwrap();

        let k_before = 100_000u128 * 200_000u128;

        env.set_sender(BOB);
        let resp = amm.swap_token_for_norn(&env.ctx(), 0, 2_000, 0).unwrap();
        let norn_out: u128 = from_response(&resp).unwrap();
        assert!(norn_out > 0);

        let resp = amm.get_pool(&env.ctx(), 0).unwrap();
        let pool: Pool = from_response(&resp).unwrap();
        let k_after = pool.reserve_norn * pool.reserve_token;
        assert!(k_after >= k_before);
    }

    #[test]
    fn test_swap_min_out_slippage() {
        let (env, mut amm) = setup();
        amm.create_pool(&env.ctx(), TOKEN_A, 100_000, 200_000)
            .unwrap();

        env.set_sender(BOB);
        // Request absurdly high minimum — should fail
        let err = amm
            .swap_norn_for_token(&env.ctx(), 0, 1_000, 999_999)
            .unwrap_err();
        assert_err_contains(&err, "slippage");
    }

    #[test]
    fn test_swap_zero_amount() {
        let (env, mut amm) = setup();
        amm.create_pool(&env.ctx(), TOKEN_A, 100_000, 200_000)
            .unwrap();

        let err = amm.swap_norn_for_token(&env.ctx(), 0, 0, 0).unwrap_err();
        assert_err_contains(&err, "positive");

        let err = amm.swap_token_for_norn(&env.ctx(), 0, 0, 0).unwrap_err();
        assert_err_contains(&err, "positive");
    }

    #[test]
    fn test_create_duplicate_pool() {
        let (env, mut amm) = setup();
        amm.create_pool(&env.ctx(), TOKEN_A, 10_000, 20_000)
            .unwrap();

        let err = amm
            .create_pool(&env.ctx(), TOKEN_A, 5_000, 10_000)
            .unwrap_err();
        assert_err_contains(&err, "already exists");
    }

    #[test]
    fn test_remove_all_liquidity() {
        let (env, mut amm) = setup();
        amm.create_pool(&env.ctx(), TOKEN_A, 10_000, 20_000)
            .unwrap();

        let resp = amm.get_lp_balance(&env.ctx(), 0, ALICE).unwrap();
        let total_lp: u128 = from_response(&resp).unwrap();

        amm.remove_liquidity(&env.ctx(), 0, total_lp).unwrap();

        let resp = amm.get_pool(&env.ctx(), 0).unwrap();
        let pool: Pool = from_response(&resp).unwrap();
        assert_eq!(pool.reserve_norn, 0);
        assert_eq!(pool.reserve_token, 0);

        let resp = amm.get_lp_balance(&env.ctx(), 0, ALICE).unwrap();
        let bal: u128 = from_response(&resp).unwrap();
        assert_eq!(bal, 0);
    }

    #[test]
    fn test_swap_large_amounts() {
        let (env, mut amm) = setup();
        // 1 billion reserves each
        let big = 1_000_000_000_000u128;
        amm.create_pool(&env.ctx(), TOKEN_A, big, big).unwrap();

        env.set_sender(BOB);
        let resp = amm
            .swap_norn_for_token(&env.ctx(), 0, 1_000_000, 0)
            .unwrap();
        let out: u128 = from_response(&resp).unwrap();
        assert!(out > 0);
        assert!(out < 1_000_000); // should get slightly less due to price impact + fee
    }

    #[test]
    fn test_fee_update_owner_only() {
        let (env, mut amm) = setup();

        // Owner can update
        amm.set_fee_bps(&env.ctx(), 50).unwrap();

        let resp = amm.get_config(&env.ctx()).unwrap();
        let (fee, _owner): (u16, Address) = from_response(&resp).unwrap();
        assert_eq!(fee, 50);

        // Non-owner cannot
        env.set_sender(BOB);
        let err = amm.set_fee_bps(&env.ctx(), 100).unwrap_err();
        assert_err_contains(&err, "only owner");

        // Cannot exceed 10%
        env.set_sender(ALICE);
        let err = amm.set_fee_bps(&env.ctx(), 1001).unwrap_err();
        assert_err_contains(&err, "exceed 10%");
    }

    #[test]
    fn test_get_quote() {
        let (env, mut amm) = setup();
        amm.create_pool(&env.ctx(), TOKEN_A, 100_000, 200_000)
            .unwrap();

        // Get quote
        let resp = amm.get_quote(&env.ctx(), 0, true, 1_000).unwrap();
        let quote: u128 = from_response(&resp).unwrap();

        // Actually swap and compare
        env.set_sender(BOB);
        let resp = amm.swap_norn_for_token(&env.ctx(), 0, 1_000, 0).unwrap();
        let actual: u128 = from_response(&resp).unwrap();
        assert_eq!(quote, actual);
    }

    #[test]
    fn test_multiple_pools() {
        let (env, mut amm) = setup();
        amm.create_pool(&env.ctx(), TOKEN_A, 10_000, 20_000)
            .unwrap();
        amm.create_pool(&env.ctx(), TOKEN_B, 50_000, 100_000)
            .unwrap();

        let resp = amm.get_pool_count(&env.ctx()).unwrap();
        let count: u64 = from_response(&resp).unwrap();
        assert_eq!(count, 2);

        let resp = amm.get_pool_by_token(&env.ctx(), TOKEN_B).unwrap();
        let pool: Pool = from_response(&resp).unwrap();
        assert_eq!(pool.id, 1);
        assert_eq!(pool.reserve_norn, 50_000);
    }

    #[test]
    fn test_insufficient_lp_balance() {
        let (env, mut amm) = setup();
        amm.create_pool(&env.ctx(), TOKEN_A, 10_000, 20_000)
            .unwrap();

        env.set_sender(BOB); // BOB has no LP tokens
        let err = amm.remove_liquidity(&env.ctx(), 0, 100).unwrap_err();
        assert_err_contains(&err, "insufficient LP balance");
    }

    #[test]
    fn test_isqrt() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(9), 3);
        assert_eq!(isqrt(10), 3);
        assert_eq!(isqrt(100), 10);
        assert_eq!(isqrt(1_000_000), 1_000);
        // Large value
        assert_eq!(isqrt(200_000_000), 14142);
    }
}
