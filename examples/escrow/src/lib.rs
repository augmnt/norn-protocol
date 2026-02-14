//! P2P Escrow contract — demonstrates contract-derived addresses for
//! token custody, state machines, and role-based access control.

#![no_std]

extern crate alloc;

use norn_sdk::prelude::*;

// ── Storage layout ──────────────────────────────────────────────────────

const DEAL_COUNT: Item<u64> = Item::new("deal_count");
const DEALS: Map<u64, Deal> = Map::new("deals");

// ── Types ───────────────────────────────────────────────────────────────

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone, PartialEq)]
pub enum DealStatus {
    Created,
    Funded,
    Delivered,
    Completed,
    Disputed,
    Cancelled,
    Refunded,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct Deal {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub token_id: TokenId,
    pub amount: u128,
    pub description: String,
    pub status: DealStatus,
    pub created_at: u64,
    pub funded_at: u64,
    pub deadline: u64,
}

// ── Contract ────────────────────────────────────────────────────────────

#[norn_contract]
pub struct Escrow;

#[norn_contract]
impl Escrow {
    #[init]
    pub fn new(_ctx: &Context) -> Self {
        DEAL_COUNT.init(&0u64);
        Escrow
    }

    #[execute]
    pub fn create_deal(
        &mut self,
        ctx: &Context,
        seller: Address,
        token_id: TokenId,
        amount: u128,
        description: String,
        deadline: u64,
    ) -> ContractResult {
        ensure!(amount > 0, "amount must be positive");
        ensure!(description.len() <= 256, "description too long (max 256)");
        ensure!(deadline > ctx.timestamp(), "deadline must be in the future");
        ensure!(seller != ctx.sender(), "buyer and seller must differ");

        let id = DEAL_COUNT.load_or(0u64);
        let deal = Deal {
            id,
            buyer: ctx.sender(),
            seller,
            token_id,
            amount,
            description,
            status: DealStatus::Created,
            created_at: ctx.timestamp(),
            funded_at: 0,
            deadline,
        };
        DEALS.save(&id, &deal)?;
        DEAL_COUNT.save(&safe_add_u64(id, 1)?)?;

        Ok(Response::with_action("create_deal")
            .add_attribute("deal_id", format!("{}", id))
            .set_data(&id))
    }

    #[execute]
    pub fn fund_deal(&mut self, ctx: &Context, deal_id: u64) -> ContractResult {
        let mut deal = DEALS.load(&deal_id)?;
        ensure!(
            deal.status == DealStatus::Created,
            "deal is not in Created status"
        );
        ensure!(deal.buyer == ctx.sender(), "only buyer can fund");

        // Transfer tokens from buyer to contract address.
        let contract = ctx.contract_address();
        ctx.transfer(&ctx.sender(), &contract, &deal.token_id, deal.amount);

        deal.status = DealStatus::Funded;
        deal.funded_at = ctx.timestamp();
        DEALS.save(&deal_id, &deal)?;

        Ok(Response::with_action("fund_deal").add_attribute("deal_id", format!("{}", deal_id)))
    }

    #[execute]
    pub fn mark_delivered(&mut self, ctx: &Context, deal_id: u64) -> ContractResult {
        let mut deal = DEALS.load(&deal_id)?;
        ensure!(
            deal.status == DealStatus::Funded,
            "deal is not in Funded status"
        );
        ensure!(
            deal.seller == ctx.sender(),
            "only seller can mark delivered"
        );

        deal.status = DealStatus::Delivered;
        DEALS.save(&deal_id, &deal)?;

        Ok(
            Response::with_action("mark_delivered")
                .add_attribute("deal_id", format!("{}", deal_id)),
        )
    }

    #[execute]
    pub fn confirm_received(&mut self, ctx: &Context, deal_id: u64) -> ContractResult {
        let mut deal = DEALS.load(&deal_id)?;
        ensure!(
            deal.status == DealStatus::Delivered,
            "deal is not in Delivered status"
        );
        ensure!(deal.buyer == ctx.sender(), "only buyer can confirm");

        // Release funds to seller.
        ctx.transfer_from_contract(&deal.seller, &deal.token_id, deal.amount);

        deal.status = DealStatus::Completed;
        DEALS.save(&deal_id, &deal)?;

        Ok(Response::with_action("confirm_received")
            .add_attribute("deal_id", format!("{}", deal_id)))
    }

    #[execute]
    pub fn dispute(&mut self, ctx: &Context, deal_id: u64) -> ContractResult {
        let mut deal = DEALS.load(&deal_id)?;
        ensure!(
            deal.status == DealStatus::Funded || deal.status == DealStatus::Delivered,
            "can only dispute Funded or Delivered deals"
        );
        ensure!(deal.buyer == ctx.sender(), "only buyer can dispute");

        deal.status = DealStatus::Disputed;
        DEALS.save(&deal_id, &deal)?;

        Ok(Response::with_action("dispute").add_attribute("deal_id", format!("{}", deal_id)))
    }

    #[execute]
    pub fn cancel_deal(&mut self, ctx: &Context, deal_id: u64) -> ContractResult {
        let mut deal = DEALS.load(&deal_id)?;
        ensure!(
            deal.status == DealStatus::Created,
            "can only cancel Created deals"
        );
        ensure!(deal.buyer == ctx.sender(), "only buyer can cancel");

        deal.status = DealStatus::Cancelled;
        DEALS.save(&deal_id, &deal)?;

        Ok(Response::with_action("cancel_deal").add_attribute("deal_id", format!("{}", deal_id)))
    }

    #[execute]
    pub fn refund_expired(&mut self, ctx: &Context, deal_id: u64) -> ContractResult {
        let deal = DEALS.load(&deal_id)?;
        ensure!(
            deal.status == DealStatus::Funded
                || deal.status == DealStatus::Delivered
                || deal.status == DealStatus::Disputed,
            "deal is not refundable"
        );
        ensure!(
            ctx.timestamp() >= deal.deadline,
            "deadline has not passed yet"
        );

        // Refund tokens to buyer.
        ctx.transfer_from_contract(&deal.buyer, &deal.token_id, deal.amount);

        let mut deal = deal;
        deal.status = DealStatus::Refunded;
        DEALS.save(&deal_id, &deal)?;

        Ok(
            Response::with_action("refund_expired")
                .add_attribute("deal_id", format!("{}", deal_id)),
        )
    }

    #[query]
    pub fn get_deal(&self, _ctx: &Context, deal_id: u64) -> ContractResult {
        let deal = DEALS.load(&deal_id)?;
        ok(deal)
    }

    #[query]
    pub fn get_deal_count(&self, _ctx: &Context) -> ContractResult {
        let count = DEAL_COUNT.load_or(0u64);
        ok(count)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    const TOKEN: TokenId = [42u8; 32];
    const CONTRACT_ADDR: Address = [99u8; 20];

    fn setup() -> (TestEnv, Escrow) {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let escrow = Escrow::new(&env.ctx());
        (env, escrow)
    }

    fn create_deal(env: &TestEnv, escrow: &mut Escrow) -> u64 {
        let resp = escrow
            .create_deal(
                &env.ctx(),
                BOB,
                TOKEN,
                500,
                String::from("Buy widget"),
                2000,
            )
            .unwrap();
        from_response::<u64>(&resp).unwrap()
    }

    #[test]
    fn test_create_deal() {
        let (env, mut escrow) = setup();
        let id = create_deal(&env, &mut escrow);
        assert_eq!(id, 0);

        let resp = escrow.get_deal(&env.ctx(), 0).unwrap();
        let deal: Deal = from_response(&resp).unwrap();
        assert_eq!(deal.buyer, ALICE);
        assert_eq!(deal.seller, BOB);
        assert_eq!(deal.amount, 500);
        assert_eq!(deal.status, DealStatus::Created);
    }

    #[test]
    fn test_fund_deal() {
        let (env, mut escrow) = setup();
        create_deal(&env, &mut escrow);

        let resp = escrow.fund_deal(&env.ctx(), 0).unwrap();
        assert_attribute(&resp, "action", "fund_deal");

        let resp = escrow.get_deal(&env.ctx(), 0).unwrap();
        let deal: Deal = from_response(&resp).unwrap();
        assert_eq!(deal.status, DealStatus::Funded);

        // Verify transfer was recorded: buyer -> contract
        let transfers = env.transfers();
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].0, ALICE.to_vec());
        assert_eq!(transfers[0].1, CONTRACT_ADDR.to_vec());
        assert_eq!(transfers[0].3, 500);
    }

    #[test]
    fn test_full_happy_path() {
        let (env, mut escrow) = setup();
        create_deal(&env, &mut escrow);

        // Fund
        escrow.fund_deal(&env.ctx(), 0).unwrap();

        // Seller marks delivered
        env.set_sender(BOB);
        escrow.mark_delivered(&env.ctx(), 0).unwrap();

        // Buyer confirms
        env.set_sender(ALICE);
        escrow.confirm_received(&env.ctx(), 0).unwrap();

        let resp = escrow.get_deal(&env.ctx(), 0).unwrap();
        let deal: Deal = from_response(&resp).unwrap();
        assert_eq!(deal.status, DealStatus::Completed);

        // Verify transfers: fund(buyer->contract) + release(contract->seller)
        let transfers = env.transfers();
        assert_eq!(transfers.len(), 2);
        assert_eq!(transfers[1].0, CONTRACT_ADDR.to_vec());
        assert_eq!(transfers[1].1, BOB.to_vec());
        assert_eq!(transfers[1].3, 500);
    }

    #[test]
    fn test_cancel_before_funding() {
        let (env, mut escrow) = setup();
        create_deal(&env, &mut escrow);

        escrow.cancel_deal(&env.ctx(), 0).unwrap();

        let resp = escrow.get_deal(&env.ctx(), 0).unwrap();
        let deal: Deal = from_response(&resp).unwrap();
        assert_eq!(deal.status, DealStatus::Cancelled);
    }

    #[test]
    fn test_cannot_cancel_after_funding() {
        let (env, mut escrow) = setup();
        create_deal(&env, &mut escrow);
        escrow.fund_deal(&env.ctx(), 0).unwrap();

        let err = escrow.cancel_deal(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "can only cancel Created deals");
    }

    #[test]
    fn test_dispute() {
        let (env, mut escrow) = setup();
        create_deal(&env, &mut escrow);
        escrow.fund_deal(&env.ctx(), 0).unwrap();

        escrow.dispute(&env.ctx(), 0).unwrap();

        let resp = escrow.get_deal(&env.ctx(), 0).unwrap();
        let deal: Deal = from_response(&resp).unwrap();
        assert_eq!(deal.status, DealStatus::Disputed);
    }

    #[test]
    fn test_refund_expired() {
        let (env, mut escrow) = setup();
        create_deal(&env, &mut escrow);
        escrow.fund_deal(&env.ctx(), 0).unwrap();

        // Advance time past deadline
        env.set_timestamp(3000);
        escrow.refund_expired(&env.ctx(), 0).unwrap();

        let resp = escrow.get_deal(&env.ctx(), 0).unwrap();
        let deal: Deal = from_response(&resp).unwrap();
        assert_eq!(deal.status, DealStatus::Refunded);

        // Verify refund transfer: contract -> buyer
        let transfers = env.transfers();
        assert_eq!(transfers.len(), 2);
        assert_eq!(transfers[1].0, CONTRACT_ADDR.to_vec());
        assert_eq!(transfers[1].1, ALICE.to_vec());
    }

    #[test]
    fn test_cannot_refund_before_deadline() {
        let (env, mut escrow) = setup();
        create_deal(&env, &mut escrow);
        escrow.fund_deal(&env.ctx(), 0).unwrap();

        let err = escrow.refund_expired(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "deadline has not passed yet");
    }

    #[test]
    fn test_only_buyer_can_confirm() {
        let (env, mut escrow) = setup();
        create_deal(&env, &mut escrow);
        escrow.fund_deal(&env.ctx(), 0).unwrap();
        env.set_sender(BOB);
        escrow.mark_delivered(&env.ctx(), 0).unwrap();

        // Bob (seller) tries to confirm — should fail
        let err = escrow.confirm_received(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "only buyer can confirm");
    }

    #[test]
    fn test_only_seller_can_deliver() {
        let (env, mut escrow) = setup();
        create_deal(&env, &mut escrow);
        escrow.fund_deal(&env.ctx(), 0).unwrap();

        // Alice (buyer) tries to mark delivered — should fail
        let err = escrow.mark_delivered(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "only seller can mark delivered");
    }

    #[test]
    fn test_query_deal_count() {
        let (env, mut escrow) = setup();
        create_deal(&env, &mut escrow);
        create_deal(&env, &mut escrow);

        let resp = escrow.get_deal_count(&env.ctx()).unwrap();
        let count: u64 = from_response(&resp).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_create_deal_validation() {
        let (env, mut escrow) = setup();

        // Zero amount
        let err = escrow
            .create_deal(&env.ctx(), BOB, TOKEN, 0, String::from("x"), 2000)
            .unwrap_err();
        assert_err_contains(&err, "amount must be positive");

        // Deadline in the past
        let err = escrow
            .create_deal(&env.ctx(), BOB, TOKEN, 100, String::from("x"), 500)
            .unwrap_err();
        assert_err_contains(&err, "deadline must be in the future");

        // Same buyer and seller
        let err = escrow
            .create_deal(&env.ctx(), ALICE, TOKEN, 100, String::from("x"), 2000)
            .unwrap_err();
        assert_err_contains(&err, "buyer and seller must differ");
    }
}
