//! Multisig Treasury contract — shared funds requiring N-of-M approvals
//! for outgoing transfers. Supports proposals, approvals, rejections,
//! deposits, and automatic execution when threshold is met.

#![no_std]

extern crate alloc;

use norn_sdk::prelude::*;

// ── Storage layout ──────────────────────────────────────────────────────

const INITIALIZED: Item<bool> = Item::new("initialized");
const CONFIG: Item<TreasuryConfig> = Item::new("config");
const PROPOSAL_COUNT: Item<u64> = Item::new("prop_count");
const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");
const APPROVALS: Map<(u64, [u8; 20]), bool> = Map::new("approvals");

// ── Types ───────────────────────────────────────────────────────────────

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone, PartialEq)]
pub enum ProposalStatus {
    Proposed,
    Executed,
    Rejected,
    Expired,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct TreasuryConfig {
    pub name: String,
    pub owners: Vec<Address>,
    pub required_approvals: u64,
    pub created_at: u64,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub to: Address,
    pub token_id: TokenId,
    pub amount: u128,
    pub description: String,
    pub status: ProposalStatus,
    pub approval_count: u64,
    pub created_at: u64,
    pub deadline: u64,
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn is_owner(config: &TreasuryConfig, addr: &Address) -> bool {
    config.owners.iter().any(|o| o == addr)
}

fn has_duplicates(owners: &[Address]) -> bool {
    for i in 0..owners.len() {
        for j in (i + 1)..owners.len() {
            if owners[i] == owners[j] {
                return true;
            }
        }
    }
    false
}

// ── Contract ────────────────────────────────────────────────────────────

#[norn_contract]
pub struct MultisigTreasury;

#[norn_contract]
impl MultisigTreasury {
    #[init]
    pub fn new(_ctx: &Context) -> Self {
        INITIALIZED.init(&false);
        PROPOSAL_COUNT.init(&0u64);
        MultisigTreasury
    }

    #[execute]
    pub fn initialize(
        &mut self,
        ctx: &Context,
        owners: Vec<Address>,
        required_approvals: u64,
        name: String,
    ) -> ContractResult {
        let already = INITIALIZED.load_or(false);
        ensure!(!already, "already initialized");
        ensure!(owners.len() >= 2, "need at least 2 owners");
        ensure!(required_approvals >= 1, "need at least 1 approval");
        ensure!(
            required_approvals <= owners.len() as u64,
            "required_approvals exceeds owner count"
        );
        ensure!(name.len() <= 64, "name too long (max 64)");
        ensure!(!has_duplicates(&owners), "duplicate owner addresses");

        CONFIG.save(&TreasuryConfig {
            name,
            owners,
            required_approvals,
            created_at: ctx.timestamp(),
        })?;
        INITIALIZED.save(&true)?;

        Ok(Response::with_action("initialize"))
    }

    #[execute]
    pub fn propose(
        &mut self,
        ctx: &Context,
        to: Address,
        token_id: TokenId,
        amount: u128,
        description: String,
        deadline: u64,
    ) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(is_owner(&config, &ctx.sender()), "only owners can propose");
        ensure!(amount > 0, "amount must be positive");
        ensure!(description.len() <= 256, "description too long (max 256)");
        ensure!(deadline > ctx.timestamp(), "deadline must be in the future");

        let id = PROPOSAL_COUNT.load_or(0u64);
        let proposal = Proposal {
            id,
            proposer: ctx.sender(),
            to,
            token_id,
            amount,
            description,
            status: ProposalStatus::Proposed,
            approval_count: 0,
            created_at: ctx.timestamp(),
            deadline,
        };
        PROPOSALS.save(&id, &proposal)?;
        PROPOSAL_COUNT.save(&safe_add_u64(id, 1)?)?;

        Ok(Response::with_action("propose")
            .add_attribute("proposal_id", format!("{}", id))
            .set_data(&id))
    }

    #[execute]
    pub fn approve(&mut self, ctx: &Context, proposal_id: u64) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(is_owner(&config, &ctx.sender()), "only owners can approve");

        let mut proposal = PROPOSALS.load(&proposal_id)?;
        ensure!(
            proposal.status == ProposalStatus::Proposed,
            "proposal is not in Proposed status"
        );
        ensure!(
            ctx.timestamp() < proposal.deadline,
            "proposal has expired"
        );

        let key = (proposal_id, ctx.sender());
        let already = APPROVALS.load(&key).unwrap_or(false);
        ensure!(!already, "already approved");

        APPROVALS.save(&key, &true)?;
        proposal.approval_count = safe_add_u64(proposal.approval_count, 1)?;

        // Auto-execute if threshold met
        if proposal.approval_count >= config.required_approvals {
            let contract = ctx.contract_address();
            ctx.transfer(&contract, &proposal.to, &proposal.token_id, proposal.amount);
            proposal.status = ProposalStatus::Executed;
        }

        PROPOSALS.save(&proposal_id, &proposal)?;

        Ok(Response::with_action("approve")
            .add_attribute("proposal_id", format!("{}", proposal_id))
            .add_attribute("approval_count", format!("{}", proposal.approval_count)))
    }

    #[execute]
    pub fn reject(&mut self, ctx: &Context, proposal_id: u64) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(is_owner(&config, &ctx.sender()), "only owners can reject");

        let mut proposal = PROPOSALS.load(&proposal_id)?;
        ensure!(
            proposal.status == ProposalStatus::Proposed,
            "proposal is not in Proposed status"
        );

        proposal.status = ProposalStatus::Rejected;
        PROPOSALS.save(&proposal_id, &proposal)?;

        Ok(Response::with_action("reject")
            .add_attribute("proposal_id", format!("{}", proposal_id)))
    }

    #[execute]
    pub fn deposit(&mut self, ctx: &Context, token_id: TokenId, amount: u128) -> ContractResult {
        ensure!(amount > 0, "amount must be positive");

        let contract = ctx.contract_address();
        ctx.transfer(&ctx.sender(), &contract, &token_id, amount);

        Ok(Response::with_action("deposit")
            .add_attribute("amount", format!("{}", amount)))
    }

    #[execute]
    pub fn revoke_approval(&mut self, ctx: &Context, proposal_id: u64) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(
            is_owner(&config, &ctx.sender()),
            "only owners can revoke approval"
        );

        let mut proposal = PROPOSALS.load(&proposal_id)?;
        ensure!(
            proposal.status == ProposalStatus::Proposed,
            "proposal is not in Proposed status"
        );

        let key = (proposal_id, ctx.sender());
        let approved = APPROVALS.load(&key).unwrap_or(false);
        ensure!(approved, "you have not approved this proposal");

        APPROVALS.save(&key, &false)?;
        proposal.approval_count = safe_sub_u64(proposal.approval_count, 1)?;
        PROPOSALS.save(&proposal_id, &proposal)?;

        Ok(Response::with_action("revoke_approval")
            .add_attribute("proposal_id", format!("{}", proposal_id))
            .add_attribute("approval_count", format!("{}", proposal.approval_count)))
    }

    #[execute]
    pub fn expire_proposal(&mut self, ctx: &Context, proposal_id: u64) -> ContractResult {
        let mut proposal = PROPOSALS.load(&proposal_id)?;
        ensure!(
            proposal.status == ProposalStatus::Proposed,
            "proposal is not in Proposed status"
        );
        ensure!(
            ctx.timestamp() >= proposal.deadline,
            "deadline has not passed yet"
        );

        proposal.status = ProposalStatus::Expired;
        PROPOSALS.save(&proposal_id, &proposal)?;

        Ok(Response::with_action("expire_proposal")
            .add_attribute("proposal_id", format!("{}", proposal_id)))
    }

    #[query]
    pub fn get_config(&self, _ctx: &Context) -> ContractResult {
        let config = CONFIG.load()?;
        ok(config)
    }

    #[query]
    pub fn get_proposal(&self, _ctx: &Context, proposal_id: u64) -> ContractResult {
        let proposal = PROPOSALS.load(&proposal_id)?;
        ok(proposal)
    }

    #[query]
    pub fn get_proposal_count(&self, _ctx: &Context) -> ContractResult {
        let count = PROPOSAL_COUNT.load_or(0u64);
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
    const CHARLIE: Address = [3u8; 20];

    fn setup() -> (TestEnv, MultisigTreasury) {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let mut treasury = MultisigTreasury::new(&env.ctx());
        treasury
            .initialize(
                &env.ctx(),
                vec![ALICE, BOB],
                2,
                String::from("Team Treasury"),
            )
            .unwrap();
        (env, treasury)
    }

    fn create_proposal(env: &TestEnv, treasury: &mut MultisigTreasury) -> u64 {
        let resp = treasury
            .propose(
                &env.ctx(),
                CHARLIE,
                TOKEN,
                1000,
                String::from("Pay Charlie"),
                2000,
            )
            .unwrap();
        from_response::<u64>(&resp).unwrap()
    }

    #[test]
    fn test_init() {
        let (env, treasury) = setup();
        let resp = treasury.get_config(&env.ctx()).unwrap();
        let config: TreasuryConfig = from_response(&resp).unwrap();
        assert_eq!(config.name, "Team Treasury");
        assert_eq!(config.owners.len(), 2);
        assert_eq!(config.required_approvals, 2);
    }

    #[test]
    fn test_init_min_owners() {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let mut treasury = MultisigTreasury::new(&env.ctx());
        let err = treasury
            .initialize(&env.ctx(), vec![ALICE], 1, String::from("Solo"))
            .unwrap_err();
        assert_err_contains(&err, "need at least 2 owners");
    }

    #[test]
    fn test_init_duplicate_owners() {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let mut treasury = MultisigTreasury::new(&env.ctx());
        let err = treasury
            .initialize(&env.ctx(), vec![ALICE, ALICE], 1, String::from("Dup"))
            .unwrap_err();
        assert_err_contains(&err, "duplicate owner addresses");
    }

    #[test]
    fn test_propose() {
        let (env, mut treasury) = setup();
        let id = create_proposal(&env, &mut treasury);
        assert_eq!(id, 0);

        let resp = treasury.get_proposal(&env.ctx(), 0).unwrap();
        let proposal: Proposal = from_response(&resp).unwrap();
        assert_eq!(proposal.proposer, ALICE);
        assert_eq!(proposal.to, CHARLIE);
        assert_eq!(proposal.amount, 1000);
        assert_eq!(proposal.status, ProposalStatus::Proposed);
        assert_eq!(proposal.approval_count, 0);
    }

    #[test]
    fn test_non_owner_cannot_propose() {
        let (env, mut treasury) = setup();
        env.set_sender(CHARLIE);
        let err = treasury
            .propose(
                &env.ctx(),
                BOB,
                TOKEN,
                100,
                String::from("sneaky"),
                2000,
            )
            .unwrap_err();
        assert_err_contains(&err, "only owners can propose");
    }

    #[test]
    fn test_single_approval() {
        let (env, mut treasury) = setup();
        create_proposal(&env, &mut treasury);

        treasury.approve(&env.ctx(), 0).unwrap();

        let resp = treasury.get_proposal(&env.ctx(), 0).unwrap();
        let proposal: Proposal = from_response(&resp).unwrap();
        assert_eq!(proposal.approval_count, 1);
        assert_eq!(proposal.status, ProposalStatus::Proposed);
    }

    #[test]
    fn test_threshold_met_auto_execute() {
        let (env, mut treasury) = setup();
        create_proposal(&env, &mut treasury);

        // Alice approves
        treasury.approve(&env.ctx(), 0).unwrap();

        // Bob approves — threshold met
        env.set_sender(BOB);
        treasury.approve(&env.ctx(), 0).unwrap();

        let resp = treasury.get_proposal(&env.ctx(), 0).unwrap();
        let proposal: Proposal = from_response(&resp).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Executed);
        assert_eq!(proposal.approval_count, 2);

        // Verify transfer was recorded: contract -> CHARLIE
        let transfers = env.transfers();
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].0, CONTRACT_ADDR.to_vec());
        assert_eq!(transfers[0].1, CHARLIE.to_vec());
        assert_eq!(transfers[0].3, 1000);
    }

    #[test]
    fn test_cannot_approve_twice() {
        let (env, mut treasury) = setup();
        create_proposal(&env, &mut treasury);

        treasury.approve(&env.ctx(), 0).unwrap();
        let err = treasury.approve(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "already approved");
    }

    #[test]
    fn test_reject() {
        let (env, mut treasury) = setup();
        create_proposal(&env, &mut treasury);

        treasury.reject(&env.ctx(), 0).unwrap();

        let resp = treasury.get_proposal(&env.ctx(), 0).unwrap();
        let proposal: Proposal = from_response(&resp).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_cannot_approve_rejected() {
        let (env, mut treasury) = setup();
        create_proposal(&env, &mut treasury);

        treasury.reject(&env.ctx(), 0).unwrap();

        let err = treasury.approve(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "proposal is not in Proposed status");
    }

    #[test]
    fn test_revoke_approval() {
        let (env, mut treasury) = setup();
        create_proposal(&env, &mut treasury);

        treasury.approve(&env.ctx(), 0).unwrap();

        let resp = treasury.get_proposal(&env.ctx(), 0).unwrap();
        let p: Proposal = from_response(&resp).unwrap();
        assert_eq!(p.approval_count, 1);

        treasury.revoke_approval(&env.ctx(), 0).unwrap();

        let resp = treasury.get_proposal(&env.ctx(), 0).unwrap();
        let p: Proposal = from_response(&resp).unwrap();
        assert_eq!(p.approval_count, 0);
    }

    #[test]
    fn test_expire_proposal() {
        let (env, mut treasury) = setup();
        create_proposal(&env, &mut treasury);

        // Before deadline — should fail
        let err = treasury.expire_proposal(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "deadline has not passed yet");

        // After deadline
        env.set_timestamp(3000);
        treasury.expire_proposal(&env.ctx(), 0).unwrap();

        let resp = treasury.get_proposal(&env.ctx(), 0).unwrap();
        let proposal: Proposal = from_response(&resp).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Expired);
    }

    #[test]
    fn test_deposit() {
        let (env, mut treasury) = setup();
        env.set_sender(CHARLIE); // Anyone can deposit
        treasury.deposit(&env.ctx(), TOKEN, 5000).unwrap();

        let transfers = env.transfers();
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].0, CHARLIE.to_vec());
        assert_eq!(transfers[0].1, CONTRACT_ADDR.to_vec());
        assert_eq!(transfers[0].3, 5000);
    }

    #[test]
    fn test_proposal_count() {
        let (env, mut treasury) = setup();
        create_proposal(&env, &mut treasury);
        create_proposal(&env, &mut treasury);

        let resp = treasury.get_proposal_count(&env.ctx()).unwrap();
        let count: u64 = from_response(&resp).unwrap();
        assert_eq!(count, 2);
    }
}
