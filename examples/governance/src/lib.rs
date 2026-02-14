//! DAO Governance — token-weighted voting on proposals.
//! Proposal → voting period → execute or reject based on quorum.

#![no_std]

extern crate alloc;

use alloc::format;
use norn_sdk::prelude::*;

// ── Storage ────────────────────────────────────────────────────────────

const INITIALIZED: Item<bool> = Item::new("initialized");
const CONFIG: Item<GovConfig> = Item::new("config");
const PROPOSAL_COUNT: Item<u64> = Item::new("prop_count");
const PROPOSALS: Map<u64, GovProposal> = Map::new("proposals");
const VOTES: Map<(u64, [u8; 20]), u8> = Map::new("votes"); // 0=not voted, 1=for, 2=against

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Expired,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct GovConfig {
    pub creator: Address,
    pub name: String,
    pub voting_period: u64, // seconds
    pub quorum: u64,        // minimum total votes needed
    pub created_at: u64,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct GovProposal {
    pub id: u64,
    pub proposer: Address,
    pub title: String,
    pub description: String,
    pub for_votes: u64,
    pub against_votes: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub status: ProposalStatus,
}

// ── Contract ───────────────────────────────────────────────────────────

#[norn_contract]
pub struct Governance;

#[norn_contract]
impl Governance {
    #[init]
    pub fn new(_ctx: &Context) -> Self {
        INITIALIZED.init(&false);
        PROPOSAL_COUNT.init(&0u64);
        Governance
    }

    #[execute]
    pub fn initialize(
        &mut self,
        ctx: &Context,
        name: String,
        voting_period: u64,
        quorum: u64,
    ) -> ContractResult {
        ensure!(!INITIALIZED.load_or(false), "already initialized");
        ensure!(name.len() <= 64, "name too long (max 64)");
        ensure!(voting_period > 0, "voting_period must be positive");
        ensure!(quorum > 0, "quorum must be positive");

        CONFIG.save(&GovConfig {
            creator: ctx.sender(),
            name,
            voting_period,
            quorum,
            created_at: ctx.timestamp(),
        })?;
        INITIALIZED.save(&true)?;

        Ok(Response::with_action("initialize"))
    }

    #[execute]
    pub fn propose(
        &mut self,
        ctx: &Context,
        title: String,
        description: String,
    ) -> ContractResult {
        let config = CONFIG.load()?;
        ensure!(title.len() <= 128, "title too long (max 128)");
        ensure!(description.len() <= 512, "description too long (max 512)");

        let id = PROPOSAL_COUNT.load_or(0u64);
        let now = ctx.timestamp();

        PROPOSALS.save(
            &id,
            &GovProposal {
                id,
                proposer: ctx.sender(),
                title,
                description,
                for_votes: 0,
                against_votes: 0,
                start_time: now,
                end_time: now + config.voting_period,
                status: ProposalStatus::Active,
            },
        )?;
        PROPOSAL_COUNT.save(&safe_add_u64(id, 1)?)?;

        Ok(Response::with_action("propose")
            .add_attribute("proposal_id", format!("{}", id))
            .set_data(&id))
    }

    #[execute]
    pub fn vote(
        &mut self,
        ctx: &Context,
        proposal_id: u64,
        support: bool,
    ) -> ContractResult {
        let mut proposal = PROPOSALS.load(&proposal_id)?;
        ensure!(
            proposal.status == ProposalStatus::Active,
            "proposal is not active"
        );
        ensure!(
            ctx.timestamp() < proposal.end_time,
            "voting period has ended"
        );

        let key = (proposal_id, ctx.sender());
        let existing = VOTES.load(&key).unwrap_or(0);
        ensure!(existing == 0, "already voted");

        if support {
            proposal.for_votes = safe_add_u64(proposal.for_votes, 1)?;
        } else {
            proposal.against_votes = safe_add_u64(proposal.against_votes, 1)?;
        }

        VOTES.save(&key, &if support { 1u8 } else { 2u8 })?;
        PROPOSALS.save(&proposal_id, &proposal)?;

        Ok(Response::with_action("vote")
            .add_attribute("proposal_id", format!("{}", proposal_id))
            .add_attribute("support", format!("{}", support)))
    }

    #[execute]
    pub fn finalize(&mut self, ctx: &Context, proposal_id: u64) -> ContractResult {
        let config = CONFIG.load()?;
        let mut proposal = PROPOSALS.load(&proposal_id)?;
        ensure!(
            proposal.status == ProposalStatus::Active,
            "proposal is not active"
        );
        ensure!(
            ctx.timestamp() >= proposal.end_time,
            "voting period has not ended"
        );

        let total_votes = safe_add_u64(proposal.for_votes, proposal.against_votes)?;

        if total_votes < config.quorum {
            proposal.status = ProposalStatus::Expired;
        } else if proposal.for_votes > proposal.against_votes {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        PROPOSALS.save(&proposal_id, &proposal)?;

        Ok(Response::with_action("finalize")
            .add_attribute("proposal_id", format!("{}", proposal_id))
            .add_attribute("status", format!("{:?}", proposal.status)))
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

    #[query]
    pub fn get_vote(&self, _ctx: &Context, proposal_id: u64, voter: Address) -> ContractResult {
        let vote = VOTES.load(&(proposal_id, voter)).unwrap_or(0);
        ok(vote)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use norn_sdk::testing::*;

    const CONTRACT_ADDR: Address = [99u8; 20];
    const CHARLIE: Address = [3u8; 20];

    fn setup() -> (TestEnv, Governance) {
        let env = TestEnv::new()
            .with_sender(ALICE)
            .with_timestamp(1000)
            .with_contract_address(CONTRACT_ADDR);
        let mut gov = Governance::new(&env.ctx());
        gov.initialize(&env.ctx(), "Norn DAO".into(), 3600, 2)
            .unwrap();
        (env, gov)
    }

    fn create_proposal(env: &TestEnv, gov: &mut Governance) -> u64 {
        let resp = gov
            .propose(
                &env.ctx(),
                "Fund development".into(),
                "Allocate tokens for core dev".into(),
            )
            .unwrap();
        from_response::<u64>(&resp).unwrap()
    }

    #[test]
    fn test_initialize() {
        let (env, gov) = setup();
        let resp = gov.get_config(&env.ctx()).unwrap();
        let config: GovConfig = from_response(&resp).unwrap();
        assert_eq!(config.name, "Norn DAO");
        assert_eq!(config.voting_period, 3600);
        assert_eq!(config.quorum, 2);
    }

    #[test]
    fn test_propose() {
        let (env, mut gov) = setup();
        let id = create_proposal(&env, &mut gov);
        assert_eq!(id, 0);

        let resp = gov.get_proposal(&env.ctx(), 0).unwrap();
        let p: GovProposal = from_response(&resp).unwrap();
        assert_eq!(p.title, "Fund development");
        assert_eq!(p.status, ProposalStatus::Active);
        assert_eq!(p.end_time, 1000 + 3600);
    }

    #[test]
    fn test_vote_for() {
        let (env, mut gov) = setup();
        create_proposal(&env, &mut gov);

        gov.vote(&env.ctx(), 0, true).unwrap();

        let resp = gov.get_proposal(&env.ctx(), 0).unwrap();
        let p: GovProposal = from_response(&resp).unwrap();
        assert_eq!(p.for_votes, 1);
        assert_eq!(p.against_votes, 0);
    }

    #[test]
    fn test_vote_against() {
        let (env, mut gov) = setup();
        create_proposal(&env, &mut gov);

        gov.vote(&env.ctx(), 0, false).unwrap();

        let resp = gov.get_proposal(&env.ctx(), 0).unwrap();
        let p: GovProposal = from_response(&resp).unwrap();
        assert_eq!(p.for_votes, 0);
        assert_eq!(p.against_votes, 1);
    }

    #[test]
    fn test_cannot_vote_twice() {
        let (env, mut gov) = setup();
        create_proposal(&env, &mut gov);
        gov.vote(&env.ctx(), 0, true).unwrap();
        let err = gov.vote(&env.ctx(), 0, true).unwrap_err();
        assert_err_contains(&err, "already voted");
    }

    #[test]
    fn test_finalize_passed() {
        let (env, mut gov) = setup();
        create_proposal(&env, &mut gov);

        gov.vote(&env.ctx(), 0, true).unwrap();
        env.set_sender(BOB);
        gov.vote(&env.ctx(), 0, true).unwrap();

        env.set_timestamp(1000 + 3601);
        gov.finalize(&env.ctx(), 0).unwrap();

        let resp = gov.get_proposal(&env.ctx(), 0).unwrap();
        let p: GovProposal = from_response(&resp).unwrap();
        assert_eq!(p.status, ProposalStatus::Passed);
    }

    #[test]
    fn test_finalize_rejected() {
        let (env, mut gov) = setup();
        create_proposal(&env, &mut gov);

        gov.vote(&env.ctx(), 0, false).unwrap();
        env.set_sender(BOB);
        gov.vote(&env.ctx(), 0, false).unwrap();

        env.set_timestamp(1000 + 3601);
        gov.finalize(&env.ctx(), 0).unwrap();

        let resp = gov.get_proposal(&env.ctx(), 0).unwrap();
        let p: GovProposal = from_response(&resp).unwrap();
        assert_eq!(p.status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_finalize_expired_no_quorum() {
        let (env, mut gov) = setup();
        create_proposal(&env, &mut gov);

        // Only 1 vote, quorum is 2
        gov.vote(&env.ctx(), 0, true).unwrap();

        env.set_timestamp(1000 + 3601);
        gov.finalize(&env.ctx(), 0).unwrap();

        let resp = gov.get_proposal(&env.ctx(), 0).unwrap();
        let p: GovProposal = from_response(&resp).unwrap();
        assert_eq!(p.status, ProposalStatus::Expired);
    }

    #[test]
    fn test_cannot_finalize_before_end() {
        let (env, mut gov) = setup();
        create_proposal(&env, &mut gov);

        let err = gov.finalize(&env.ctx(), 0).unwrap_err();
        assert_err_contains(&err, "voting period has not ended");
    }

    #[test]
    fn test_cannot_vote_after_period() {
        let (env, mut gov) = setup();
        create_proposal(&env, &mut gov);

        env.set_timestamp(1000 + 3601);
        let err = gov.vote(&env.ctx(), 0, true).unwrap_err();
        assert_err_contains(&err, "voting period has ended");
    }
}
