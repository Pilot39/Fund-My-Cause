#![cfg(test)]
#![allow(deprecated)]

use super::*;
use crate::types::Category;
use crate::{CrowdfundContract, CrowdfundContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

fn setup_contract(
    env: &Env,
    deadline: u64,
    goal: i128,
    min_contribution: i128,
) -> (
    Address,
    Address,
    CrowdfundContractClient<'_>,
    token::StellarAssetClient<'_>,
) {
    env.mock_all_auths();

    let creator = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract(token_admin.clone());
    let token_admin_client = token::StellarAssetClient::new(env, &token_id);

    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(env, &contract_id);

    client.initialize(
        &creator,
        &token_id,
        &goal,
        &deadline,
        &min_contribution,
        &0i128,
        &String::from_str(env, "My Title"),
        &String::from_str(env, "My Description"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );

    (creator, token_id, client, token_admin_client)
}

#[test]
fn initialize_and_contribute_updates_state() {
    let env = Env::default();
    let deadline = 1_000u64;
    let goal = 10_000i128;
    let min_contribution = 100i128;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, goal, min_contribution);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &500);

    client.contribute(&contributor, &500, &token_id, &None);

    assert_eq!(client.total_raised(), 500);
    assert_eq!(client.contribution(&contributor), 500);
    assert!(client.is_contributor(&contributor));

    let stats = client.get_stats();
    assert_eq!(stats.total_raised, 500);
    assert_eq!(stats.goal, goal);
    assert_eq!(stats.contributor_count, 1);
    assert_eq!(stats.average_contribution, 500);
    assert_eq!(stats.largest_contribution, 500);
}

#[test]
fn cancel_allows_refund_before_deadline() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 10_000, 100);

    let contributor = Address::generate(&env);
    let token_client = token::Client::new(&env, &token_id);
    token_admin_client.mint(&contributor, &500);

    client.contribute(&contributor, &500, &token_id, &None);
    client.cancel_campaign();

    env.ledger().set_timestamp(deadline - 10);
    client.refund_single(&contributor);

    assert_eq!(client.contribution(&contributor), 0);
    assert_eq!(token_client.balance(&contributor), 500);
}

#[test]
fn invalid_platform_fee_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let result = client.try_initialize(
        &creator,
        &token_id,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &String::from_str(&env, "My Title"),
        &String::from_str(&env, "My Description"),
        &None,
        &Some(PlatformConfig {
            address: Address::generate(&env),
            fee_bps: 10_001,
        }),
        &None,
        &Category::Other,
        &None,
        &None,
    );

    assert_eq!(result.err(), Some(Ok(ContractError::InvalidFee)));
}

// ── Boundary tests (#107) ─────────────────────────────────────────────────────

#[test]
fn accepted_token_whitelist_is_enforced() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let allowed_token = env.register_stellar_asset_contract(token_admin.clone());
    let other_token = env.register_stellar_asset_contract(token_admin);
    let allowed_token_admin = token::StellarAssetClient::new(&env, &allowed_token);

    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let mut accepted_tokens = Vec::new(&env);
    accepted_tokens.push_back(allowed_token.clone());

    client.initialize(
        &creator,
        &allowed_token,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &String::from_str(&env, "My Title"),
        &String::from_str(&env, "My Description"),
        &None,
        &None,
        &Some(accepted_tokens),
        &Category::Other,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    allowed_token_admin.mint(&contributor, &100);

    let result = client.try_contribute(&contributor, &100, &other_token, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::TokenNotAccepted)));
}

// ── refund_batch tests (#278) ─────────────────────────────────────────────────

#[test]
fn refund_batch_refunds_multiple_contributors() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let token_client = token::Client::new(&env, &token_id);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);
    let c3 = Address::generate(&env);

    token_admin_client.mint(&c1, &500);
    token_admin_client.mint(&c2, &300);
    token_admin_client.mint(&c3, &200);

    client.contribute(&c1, &500, &token_id, &None);
    client.contribute(&c2, &300, &token_id, &None);
    client.contribute(&c3, &200, &token_id, &None);

    client.cancel_campaign();

    let mut batch = Vec::new(&env);
    batch.push_back(c1.clone());
    batch.push_back(c2.clone());
    batch.push_back(c3.clone());

    let refunded = client.refund_batch(&batch);
    assert_eq!(refunded, 3);

    assert_eq!(token_client.balance(&c1), 500);
    assert_eq!(token_client.balance(&c2), 300);
    assert_eq!(token_client.balance(&c3), 200);
    assert_eq!(client.contribution(&c1), 0);
    assert_eq!(client.contribution(&c2), 0);
    assert_eq!(client.contribution(&c3), 0);
}

#[test]
fn refund_batch_skips_already_refunded() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let c1 = Address::generate(&env);
    token_admin_client.mint(&c1, &500);
    client.contribute(&c1, &500, &token_id, &None);
    client.cancel_campaign();

    let mut batch = Vec::new(&env);
    batch.push_back(c1.clone());
    let r1 = client.refund_batch(&batch);
    assert_eq!(r1, 1);

    let r2 = client.refund_batch(&batch);
    assert_eq!(r2, 0);
}

#[test]
fn refund_batch_fails_when_campaign_still_active() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let c1 = Address::generate(&env);
    token_admin_client.mint(&c1, &500);
    client.contribute(&c1, &500, &token_id, &None);

    let mut batch = Vec::new(&env);
    batch.push_back(c1.clone());

    let result = client.try_refund_batch(&batch);
    assert_eq!(result.err(), Some(Ok(ContractError::CampaignStillActive)));
}

// ── pause/unpause tests (#279) ────────────────────────────────────────────────

#[test]
fn pause_blocks_contributions_and_unpause_resumes() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &1_000);

    client.pause();
    assert_eq!(client.status(), Status::Paused);

    let result = client.try_contribute(&contributor, &500, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::CampaignPaused)));

    client.unpause();
    assert_eq!(client.status(), Status::Active);

    client.contribute(&contributor, &500, &token_id, &None);
    assert_eq!(client.total_raised(), 500);
}

#[test]
fn pause_allows_refunds_when_cancelled() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let contributor = Address::generate(&env);
    let token_client = token::Client::new(&env, &token_id);
    token_admin_client.mint(&contributor, &500);

    client.contribute(&contributor, &500, &token_id, &None);
    client.cancel_campaign();

    client.refund_single(&contributor);
    assert_eq!(token_client.balance(&contributor), 500);
}

#[test]
fn unpause_fails_when_not_paused() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, _token_id, client, _) = setup_contract(&env, deadline, 100_000, 100);

    let result = client.try_unpause();
    assert_eq!(result.err(), Some(Ok(ContractError::NotActive)));
}

#[test]
fn pause_fails_when_not_active() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, _token_id, client, _) = setup_contract(&env, deadline, 100_000, 100);

    client.cancel_campaign();

    let result = client.try_pause();
    assert_eq!(result.err(), Some(Ok(ContractError::NotActive)));
}

// ── max_contribution tests ────────────────────────────────────────────────────

fn setup_contract_with_max(
    env: &Env,
    deadline: u64,
    goal: i128,
    min_contribution: i128,
    max_contribution: i128,
) -> (
    Address,
    Address,
    CrowdfundContractClient<'_>,
    token::StellarAssetClient<'_>,
) {
    env.mock_all_auths();

    let creator = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract(token_admin.clone());
    let token_admin_client = token::StellarAssetClient::new(env, &token_id);

    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(env, &contract_id);

    client.initialize(
        &creator,
        &token_id,
        &goal,
        &deadline,
        &min_contribution,
        &max_contribution,
        &String::from_str(env, "My Title"),
        &String::from_str(env, "My Description"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );

    (creator, token_id, client, token_admin_client)
}

#[test]
fn contribute_within_max_succeeds() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract_with_max(&env, 1_000, 10_000, 100, 500);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &500);

    client.contribute(&contributor, &500, &token_id, &None);
    assert_eq!(client.contribution(&contributor), 500);
}

#[test]
fn contribute_exceeding_max_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract_with_max(&env, 1_000, 10_000, 100, 500);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &600);

    let result = client.try_contribute(&contributor, &600, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::ExceedsMaximum)));
}

#[test]
fn cumulative_contribution_exceeding_max_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract_with_max(&env, 1_000, 10_000, 100, 500);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &600);

    client.contribute(&contributor, &300, &token_id, &None);
    let result = client.try_contribute(&contributor, &300, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::ExceedsMaximum)));
}

#[test]
fn no_max_limit_allows_large_contribution() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) = setup_contract(&env, 1_000, 10_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &9_000);

    client.contribute(&contributor, &9_000, &token_id, &None);
    assert_eq!(client.contribution(&contributor), 9_000);
}

#[test]
fn max_contribution_view_returns_stored_value() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract_with_max(&env, 1_000, 10_000, 100, 750);

    assert_eq!(client.max_contribution(), 750);
}

#[test]
fn initialize_with_max_below_min_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let result = client.try_initialize(
        &creator,
        &token_id,
        &10_000,
        &1_000,
        &200,
        &100, // max < min — invalid
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Desc"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::ExceedsMaximum)));
}

// ── Input validation tests ────────────────────────────────────────────────────

#[test]
fn initialize_with_empty_title_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let result = client.try_initialize(
        &creator,
        &token_id,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &String::from_str(&env, ""), // empty title
        &String::from_str(&env, "Description"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::StringEmpty)));
}

#[test]
fn initialize_with_title_too_long_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    // 65-character title (max is 64)
    let long_title = String::from_str(
        &env,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    );
    let result = client.try_initialize(
        &creator,
        &token_id,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &long_title,
        &String::from_str(&env, "Description"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::StringTooLong)));
}

#[test]
fn initialize_with_self_fee_address_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let result = client.try_initialize(
        &creator,
        &token_id,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Description"),
        &None,
        &Some(PlatformConfig {
            address: creator.clone(), // same as creator — invalid
            fee_bps: 100,
        }),
        &None,
        &Category::Other,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::SelfFeeAddress)));
}

#[test]
fn contribute_with_zero_amount_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, _) = setup_contract(&env, 1_000, 10_000, 0);

    let contributor = Address::generate(&env);
    let result = client.try_contribute(&contributor, &0, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::AmountNotPositive)));
}

#[test]
fn contribute_with_negative_amount_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, _) = setup_contract(&env, 1_000, 10_000, 0);

    let contributor = Address::generate(&env);
    let result = client.try_contribute(&contributor, &-1, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::AmountNotPositive)));
}

#[test]
fn update_metadata_with_empty_title_is_rejected() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    let result = client.try_update_metadata(&Some(String::from_str(&env, "")), &None, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::StringEmpty)));
}

#[test]
fn update_metadata_with_valid_title_succeeds() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    client.update_metadata(&Some(String::from_str(&env, "New Title")), &None, &None);
    assert_eq!(client.title(), String::from_str(&env, "New Title"));
}

// ── #424 Emergency Withdrawal with Timelock + Multi-sig ───────────────────────

#[test]
fn test_emergency_withdrawal_basic_timelock() {
    let env = Env::default();
    let deadline = 2_000u64;

    let (creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 10_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &500);
    client.contribute(&contributor, &500, &token_id, &None);

    // Initiate with 48-hour lock (172800 seconds)
    client.initiate_emergency_withdrawal(&172_800u64);

    // Cannot execute before lock expires
    let result = client.try_execute_emergency_withdrawal();
    assert_eq!(result.err(), Some(Ok(ContractError::EmergencyLocked)));

    // Advance past lock period
    env.ledger().set_timestamp(172_801u64);
    let token_client = token::Client::new(&env, &token_id);
    let before = token_client.balance(&creator);
    client.execute_emergency_withdrawal();
    assert_eq!(token_client.balance(&creator), before + 500);
    assert_eq!(client.total_raised(), 0);
}

#[test]
fn test_emergency_withdrawal_cancel_clears_lock() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    client.initiate_emergency_withdrawal(&172_800u64);
    client.cancel_emergency_withdrawal();

    // After cancel the lock is cleared, execute should fail with EmergencyLocked
    env.ledger().set_timestamp(200_000u64);
    let result = client.try_execute_emergency_withdrawal();
    assert_eq!(result.err(), Some(Ok(ContractError::EmergencyLocked)));
}

#[test]
fn test_emergency_multisig_setup_and_approve() {
    let env = Env::default();
    let deadline = 5_000_000u64;

    let (creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 10_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &500);
    client.contribute(&contributor, &500, &token_id, &None);

    let approver1 = Address::generate(&env);
    let approver2 = Address::generate(&env);
    let mut approvers = Vec::new(&env);
    approvers.push_back(approver1.clone());
    approvers.push_back(approver2.clone());

    // Configure 2-of-2 multi-sig
    client.setup_emergency_multisig(&2u32, &approvers);

    // Initiate emergency (48-hour lock)
    client.initiate_emergency_withdrawal(&172_800u64);

    // Not enough approvals yet
    env.ledger().set_timestamp(200_000u64);
    let result = client.try_execute_emergency_withdrawal();
    assert_eq!(result.err(), Some(Ok(ContractError::MultiSigNotMet)));

    // First approver approves
    client.approve_emergency_withdrawal(&approver1);
    let result = client.try_execute_emergency_withdrawal();
    assert_eq!(result.err(), Some(Ok(ContractError::MultiSigNotMet)));

    // Second approver approves — now threshold is met
    client.approve_emergency_withdrawal(&approver2);

    let token_client = token::Client::new(&env, &token_id);
    let before = token_client.balance(&creator);
    client.execute_emergency_withdrawal();
    assert_eq!(token_client.balance(&creator), before + 500);
}

#[test]
fn test_emergency_multisig_idempotent_approval() {
    let env = Env::default();
    let deadline = 5_000_000u64;

    let (_creator, _token_id, client, _) =
        setup_contract(&env, deadline, 10_000, 100);

    let approver1 = Address::generate(&env);
    let mut approvers = Vec::new(&env);
    approvers.push_back(approver1.clone());

    // Configure 1-of-1 multi-sig
    client.setup_emergency_multisig(&1u32, &approvers);
    client.initiate_emergency_withdrawal(&172_800u64);

    // Approve twice — second call is idempotent, count stays 1
    client.approve_emergency_withdrawal(&approver1);
    client.approve_emergency_withdrawal(&approver1); // idempotent

    env.ledger().set_timestamp(200_000u64);
    // Should succeed (1 approval is enough)
    client.execute_emergency_withdrawal();
}

#[test]
fn test_emergency_multisig_reset_on_new_initiation() {
    let env = Env::default();
    let deadline = 5_000_000u64;

    let (_creator, _token_id, client, _) =
        setup_contract(&env, deadline, 10_000, 100);

    let approver1 = Address::generate(&env);
    let approver2 = Address::generate(&env);
    let mut approvers = Vec::new(&env);
    approvers.push_back(approver1.clone());
    approvers.push_back(approver2.clone());

    client.setup_emergency_multisig(&2u32, &approvers);

    // Session 1 at t=0: lock_until = 0 + 172_800 = 172_800
    client.initiate_emergency_withdrawal(&172_800u64);
    client.approve_emergency_withdrawal(&approver1);
    client.approve_emergency_withdrawal(&approver2);

    // Advance time so Session 2 gets a different lock_until value.
    // Session 2 at t=1: lock_until = 1 + 172_800 = 172_801
    // The approval count is reset to 0; per-approver stored values (172_800)
    // differ from the new lock_until (172_801), so the idempotency guard
    // would allow re-approval — but we intentionally don't re-approve here.
    env.ledger().set_timestamp(1u64);
    client.initiate_emergency_withdrawal(&172_800u64);

    // Advance past the new lock period without submitting any approvals
    env.ledger().set_timestamp(200_000u64);

    let result = client.try_execute_emergency_withdrawal();
    assert_eq!(result.err(), Some(Ok(ContractError::MultiSigNotMet)));
}

#[test]
fn test_emergency_approve_fails_for_unauthorized_address() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    let approver1 = Address::generate(&env);
    let stranger = Address::generate(&env);
    let mut approvers = Vec::new(&env);
    approvers.push_back(approver1.clone());

    client.setup_emergency_multisig(&1u32, &approvers);
    client.initiate_emergency_withdrawal(&172_800u64);

    let result = client.try_approve_emergency_withdrawal(&stranger);
    assert_eq!(result.err(), Some(Ok(ContractError::Unauthorized)));
}

#[test]
fn test_emergency_approve_fails_without_initiation() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    let approver1 = Address::generate(&env);
    let mut approvers = Vec::new(&env);
    approvers.push_back(approver1.clone());

    client.setup_emergency_multisig(&1u32, &approvers);
    // No initiation call — should fail
    let result = client.try_approve_emergency_withdrawal(&approver1);
    assert_eq!(result.err(), Some(Ok(ContractError::EmergencyLocked)));
}

// ── #425 Campaign Template System ────────────────────────────────────────────

fn make_template(env: &Env, template_type: TemplateType, suggested_min: i128) -> CampaignTemplate {
    CampaignTemplate {
        template_type,
        name: String::from_str(env, "Test Template"),
        description: String::from_str(env, "A template for testing"),
        suggested_min,
        goal_multiplier: 10_000, // 1x
    }
}

#[test]
fn test_initialize_from_template_basic() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let template = make_template(&env, TemplateType::Charity, 500);

    client.initialize_from_template(
        &creator,
        &token_id,
        &10_000i128,
        &1_000u64,
        &0i128,
        &String::from_str(&env, "Charity Campaign"),
        &String::from_str(&env, "Help us fund a good cause"),
        &template,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    assert_eq!(client.status(), Status::Active);
    assert_eq!(client.goal(), 10_000);
    // min_contribution is derived from template.suggested_min
    assert_eq!(client.min_contribution(), 500);
    // Category is mapped from TemplateType::Charity → Category::Charity
    assert_eq!(client.get_category(), Category::Charity);
}

#[test]
fn test_initialize_from_template_derives_category_from_type() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);

    let cases: &[(TemplateType, Category)] = &[
        (TemplateType::Charity, Category::Charity),
        (TemplateType::Product, Category::Technology),
        (TemplateType::Event, Category::Event),
        (TemplateType::Personal, Category::Personal),
        (TemplateType::Custom, Category::Other),
    ];

    for (template_type, expected_category) in cases {
        let contract_id = env.register_contract(None, CrowdfundContract);
        let client = CrowdfundContractClient::new(&env, &contract_id);

        let template = make_template(&env, *template_type, 100);
        client.initialize_from_template(
            &creator,
            &token_id,
            &10_000i128,
            &1_000u64,
            &0i128,
            &String::from_str(&env, "Campaign"),
            &String::from_str(&env, "Description"),
            &template,
            &None,
            &None,
            &None,
            &None,
            &None,
        );
        assert_eq!(client.get_category(), *expected_category);
    }
}

#[test]
fn test_initialize_from_template_invalid_goal_multiplier() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let mut template = make_template(&env, TemplateType::Custom, 100);
    template.goal_multiplier = 0; // invalid

    let result = client.try_initialize_from_template(
        &creator,
        &token_id,
        &10_000i128,
        &1_000u64,
        &0i128,
        &String::from_str(&env, "Campaign"),
        &String::from_str(&env, "Description"),
        &template,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::InvalidTemplate)));
}

#[test]
fn test_initialize_from_template_already_initialized() {
    let env = Env::default();
    let (_creator, token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    let creator2 = Address::generate(&env);
    let template = make_template(&env, TemplateType::Charity, 100);

    let result = client.try_initialize_from_template(
        &creator2,
        &token_id,
        &10_000i128,
        &2_000u64,
        &0i128,
        &String::from_str(&env, "Second"),
        &String::from_str(&env, "Second desc"),
        &template,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::AlreadyInitialized)));
}

#[test]
fn test_set_and_get_template() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    client.set_template(
        &TemplateType::Product,
        &String::from_str(&env, "Product Launch"),
        &String::from_str(&env, "Launch your product"),
        &1_000i128,
        &15_000u32,
    );

    let tmpl = client.get_template().unwrap();
    assert_eq!(tmpl.template_type, TemplateType::Product);
    assert_eq!(tmpl.suggested_min, 1_000);
    assert_eq!(tmpl.goal_multiplier, 15_000);
}

// ── #426 Contribution Matching ────────────────────────────────────────────────

#[test]
fn test_setup_matching_stores_config() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    let sponsor = Address::generate(&env);
    client.setup_matching(&sponsor, &5_000u32, &50_000i128);

    let cfg = client.get_matching_config().unwrap();
    assert_eq!(cfg.match_ratio, 5_000);
    assert_eq!(cfg.max_match, 50_000);
    assert_eq!(client.get_total_matched(), 0);
}

#[test]
fn test_matching_applied_on_contribute() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let sponsor = Address::generate(&env);
    // 1:1 match (10 000 bps) capped at 10 000
    client.setup_matching(&sponsor, &10_000u32, &10_000i128);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &1_000);
    client.contribute(&contributor, &1_000, &token_id, &None);

    // 1_000 contributed + 1_000 matched = 2_000 total
    assert_eq!(client.total_raised(), 2_000);
    assert_eq!(client.get_total_matched(), 1_000);
}

#[test]
fn test_matching_respects_max_match_limit() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let sponsor = Address::generate(&env);
    // 1:1 match capped at only 300
    client.setup_matching(&sponsor, &10_000u32, &300i128);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &1_000);
    client.contribute(&contributor, &1_000, &token_id, &None);

    // Only 300 matched (cap reached)
    assert_eq!(client.get_total_matched(), 300);
    assert_eq!(client.total_raised(), 1_300);
}

#[test]
fn test_setup_matching_invalid_ratio() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    let sponsor = Address::generate(&env);
    // ratio > 10_000 is invalid
    let result = client.try_setup_matching(&sponsor, &10_001u32, &50_000i128);
    assert_eq!(result.err(), Some(Ok(ContractError::InvalidFee)));
}

#[test]
fn test_setup_matching_invalid_max_match() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    let sponsor = Address::generate(&env);
    let result = client.try_setup_matching(&sponsor, &5_000u32, &0i128);
    assert_eq!(result.err(), Some(Ok(ContractError::AmountNotPositive)));
}

#[test]
fn test_matching_caps_across_multiple_contributions() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let sponsor = Address::generate(&env);
    // 50% match capped at 400
    client.setup_matching(&sponsor, &5_000u32, &400i128);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);
    token_admin_client.mint(&c1, &600);
    token_admin_client.mint(&c2, &600);

    client.contribute(&c1, &600, &token_id, &None); // 600 + 300 matched = 900
    client.contribute(&c2, &600, &token_id, &None); // 600 + 100 matched (cap) = 700

    assert_eq!(client.get_total_matched(), 400);
    assert_eq!(client.total_raised(), 1_600);
}

// ── #427 Campaign Categories ──────────────────────────────────────────────────

#[test]
fn test_get_category_returns_initialized_category() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    client.initialize(
        &creator,
        &token_id,
        &10_000i128,
        &1_000u64,
        &100i128,
        &0i128,
        &String::from_str(&env, "Tech Campaign"),
        &String::from_str(&env, "Build something great"),
        &None,
        &None,
        &None,
        &Category::Technology,
        &None,
        &None,
    );

    assert_eq!(client.get_category(), Category::Technology);
}

#[test]
fn test_update_category_changes_value() {
    let env = Env::default();
    // Start with Other category
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    assert_eq!(client.get_category(), Category::Other);

    client.update_category(&Category::Creative);
    assert_eq!(client.get_category(), Category::Creative);
}

#[test]
fn test_update_category_fails_when_not_active() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    client.cancel_campaign();

    let result = client.try_update_category(&Category::Event);
    assert_eq!(result.err(), Some(Ok(ContractError::NotActive)));
}

#[test]
fn test_campaign_info_includes_category() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    client.initialize(
        &creator,
        &token_id,
        &10_000i128,
        &1_000u64,
        &100i128,
        &0i128,
        &String::from_str(&env, "Event Campaign"),
        &String::from_str(&env, "Join the event"),
        &None,
        &None,
        &None,
        &Category::Event,
        &None,
        &None,
    );

    let info = client.get_campaign_info();
    assert_eq!(info.category, Category::Event);
}

#[test]
fn test_all_category_variants_accepted() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);

    let categories = [
        Category::Charity,
        Category::Technology,
        Category::Creative,
        Category::Event,
        Category::Personal,
        Category::Other,
    ];

    for category in &categories {
        let creator = Address::generate(&env);
        let contract_id = env.register_contract(None, CrowdfundContract);
        let client = CrowdfundContractClient::new(&env, &contract_id);

        client.initialize(
            &creator,
            &token_id,
            &10_000i128,
            &1_000u64,
            &100i128,
            &0i128,
            &String::from_str(&env, "Campaign"),
            &String::from_str(&env, "Description"),
            &None,
            &None,
            &None,
            category,
            &None,
            &None,
        );

        assert_eq!(client.get_category(), *category);
    }
}
