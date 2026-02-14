//! Simple Swap / OTC Desk — post offers to trade token A for token B
//! at a fixed rate. Counterparty fills the order.

#![no_std]

extern crate alloc;

use alloc::format;
use norn_sdk::prelude::*;

// ── Storage ────────────────────────────────────────────────────────────

const ORDER_COUNT: Item<u64> = Item::new("order_count");
const ORDERS: Map<u64, SwapOrder> = Map::new("orders");

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone, PartialEq)]
pub enum OrderStatus {
    Open,
    Filled,
    Cancelled,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct SwapOrder {
    pub id: u64,
    pub creator: Address,
    pub sell_token: TokenId,
    pub sell_amount: u128,
    pub buy_token: TokenId,
    pub buy_amount: u128,
    pub status: OrderStatus,
    pub filled_by: Address,
    pub created_at: u64,
}

// ── Contract ───────────────────────────────────────────────────────────

#[norn_contract]
pub struct Swap;

#[norn_contract]
impl Swap {
    #[init]
    pub fn new(_ctx: &Context) -> Self {
        ORDER_COUNT.init(&0u64);
        Swap
    }

    #[execute]
    pub fn create_order(
        &mut self,
        ctx: &Context,
        sell_token: TokenId,
        sell_amount: u128,
        buy_token: TokenId,
        buy_amount: u128,
    ) -> ContractResult {
        ensure!(sell_amount > 0, "sell_amount must be positive");
        ensure!(buy_amount > 0, "buy_amount must be positive");

        // Lock sell tokens in contract
        let contract = ctx.contract_address();
        ctx.transfer(&ctx.sender(), &contract, &sell_token, sell_amount);

        let id = ORDER_COUNT.load_or(0u64);
        ORDERS.save(
            &id,
            &SwapOrder {
                id,
                creator: ctx.sender(),
                sell_token,
                sell_amount,
                buy_token,
                buy_amount,
                status: OrderStatus::Open,
                filled_by: ZERO_ADDRESS,
                created_at: ctx.timestamp(),
            },
        )?;
        ORDER_COUNT.save(&safe_add_u64(id, 1)?)?;

        Ok(Response::with_action("create_order")
            .add_attribute("order_id", format!("{}", id))
            .set_data(&id))
    }

    #[execute]
    pub fn fill_order(&mut self, ctx: &Context, order_id: u64) -> ContractResult {
        let mut order = ORDERS.load(&order_id)?;
        ensure!(order.status == OrderStatus::Open, "order is not open");
        ensure!(ctx.sender() != order.creator, "cannot fill own order");

        let contract = ctx.contract_address();

        // Buyer sends buy_token to contract
        ctx.transfer(&ctx.sender(), &contract, &order.buy_token, order.buy_amount);

        // Creator gets buy_token
        ctx.transfer_from_contract(&order.creator, &order.buy_token, order.buy_amount);

        // Buyer gets sell_token
        ctx.transfer_from_contract(&ctx.sender(), &order.sell_token, order.sell_amount);

        order.status = OrderStatus::Filled;
        order.filled_by = ctx.sender();
        ORDERS.save(&order_id, &order)?;

        Ok(Response::with_action("fill_order")
            .add_attribute("order_id", format!("{}", order_id)))
    }

    #[execute]
    pub fn cancel_order(&mut self, ctx: &Context, order_id: u64) -> ContractResult {
        let mut order = ORDERS.load(&order_id)?;
        ensure!(order.status == OrderStatus::Open, "order is not open");
        ensure!(ctx.sender() == order.creator, "only creator can cancel");

        // Return locked tokens
        ctx.transfer_from_contract(&order.creator, &order.sell_token, order.sell_amount);

        order.status = OrderStatus::Cancelled;
        ORDERS.save(&order_id, &order)?;

        Ok(Response::with_action("cancel_order")
            .add_attribute("order_id", format!("{}", order_id)))
    }

    #[query]
    pub fn get_order(&self, _ctx: &Context, order_id: u64) -> ContractResult {
        let order = ORDERS.load(&order_id)?;
        ok(order)
    }

    #[query]
    pub fn get_order_count(&self, _ctx: &Context) -> ContractResult {
        let count = ORDER_COUNT.load_or(0u64);
        ok(count)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    const TOKEN_A: TokenId = [1u8; 32];
    const TOKEN_B: TokenId = [2u8; 32];
    const CONTRACT_ADDR: Address = [99u8; 20];

    fn setup() -> (TestEnv, Swap) {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let swap = Swap::new(&env.ctx());
        (env, swap)
    }

    #[test]
    fn test_create_order() {
        let (env, mut swap) = setup();
        let resp = swap
            .create_order(&env.ctx(), TOKEN_A, 1000, TOKEN_B, 500)
            .unwrap();
        let id: u64 = from_response(&resp).unwrap();
        assert_eq!(id, 0);

        let resp = swap.get_order(&env.ctx(), 0).unwrap();
        let order: SwapOrder = from_response(&resp).unwrap();
        assert_eq!(order.creator, ALICE);
        assert_eq!(order.sell_amount, 1000);
        assert_eq!(order.buy_amount, 500);
        assert_eq!(order.status, OrderStatus::Open);

        // Verify tokens locked
        let transfers = env.transfers();
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].0, ALICE.to_vec());
        assert_eq!(transfers[0].1, CONTRACT_ADDR.to_vec());
    }

    #[test]
    fn test_fill_order() {
        let (env, mut swap) = setup();
        swap.create_order(&env.ctx(), TOKEN_A, 1000, TOKEN_B, 500)
            .unwrap();

        env.set_sender(BOB);
        swap.fill_order(&env.ctx(), 0).unwrap();

        let resp = swap.get_order(&env.ctx(), 0).unwrap();
        let order: SwapOrder = from_response(&resp).unwrap();
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.filled_by, BOB);

        // Transfers: lock + buyer_send + creator_gets + buyer_gets
        let transfers = env.transfers();
        assert_eq!(transfers.len(), 4);
    }

    #[test]
    fn test_cannot_fill_own_order() {
        let (env, mut swap) = setup();
        swap.create_order(&env.ctx(), TOKEN_A, 1000, TOKEN_B, 500)
            .unwrap();

        let err = swap.fill_order(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "cannot fill own order");
    }

    #[test]
    fn test_cancel_order() {
        let (env, mut swap) = setup();
        swap.create_order(&env.ctx(), TOKEN_A, 1000, TOKEN_B, 500)
            .unwrap();

        swap.cancel_order(&env.ctx(), 0).unwrap();

        let resp = swap.get_order(&env.ctx(), 0).unwrap();
        let order: SwapOrder = from_response(&resp).unwrap();
        assert_eq!(order.status, OrderStatus::Cancelled);

        // Lock + return
        let transfers = env.transfers();
        assert_eq!(transfers.len(), 2);
    }

    #[test]
    fn test_only_creator_can_cancel() {
        let (env, mut swap) = setup();
        swap.create_order(&env.ctx(), TOKEN_A, 1000, TOKEN_B, 500)
            .unwrap();

        env.set_sender(BOB);
        let err = swap.cancel_order(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "only creator can cancel");
    }

    #[test]
    fn test_cannot_fill_cancelled() {
        let (env, mut swap) = setup();
        swap.create_order(&env.ctx(), TOKEN_A, 1000, TOKEN_B, 500)
            .unwrap();
        swap.cancel_order(&env.ctx(), 0).unwrap();

        env.set_sender(BOB);
        let err = swap.fill_order(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "order is not open");
    }

    #[test]
    fn test_order_count() {
        let (env, mut swap) = setup();
        swap.create_order(&env.ctx(), TOKEN_A, 1000, TOKEN_B, 500)
            .unwrap();
        swap.create_order(&env.ctx(), TOKEN_A, 2000, TOKEN_B, 1000)
            .unwrap();

        let resp = swap.get_order_count(&env.ctx()).unwrap();
        let count: u64 = from_response(&resp).unwrap();
        assert_eq!(count, 2);
    }
}
