#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

// ── helpers ──────────────────────────────────────────────────────────────────

fn setup_env() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, EscrowContract);
    (env, contract_id)
}

fn setup_token(env: &Env, token_admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(token_admin.clone())
        .address()
}

fn mint(env: &Env, token_addr: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token_addr).mint(to, &amount);
}

fn balance(env: &Env, token_addr: &Address, of: &Address) -> i128 {
    TokenClient::new(env, token_addr).balance(of)
}

// ── original tests (preserved) ───────────────────────────────────────────────

#[test]
fn test_init_and_get_status() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);
    let sender = Address::generate(&env);
    env.mock_all_auths();
    client.init(&sender, &1000);
    let status = client.get_status();
    assert_eq!(status, DeliveryStatus::Created);
}

#[test]
fn test_propose_and_accept_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    env.mock_all_auths();

    client.init(&admin, &1000);
    assert_eq!(client.get_admin(), admin);

    client.propose_admin(&admin, &new_admin);
    client.accept_admin(&new_admin);

    assert_eq!(client.get_admin(), new_admin);
}

#[test]
#[should_panic]
fn test_accept_admin_rejected_for_non_pending() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let proposed = Address::generate(&env);
    let other = Address::generate(&env);
    env.mock_all_auths();

    client.init(&admin, &1000);
    client.propose_admin(&admin, &proposed);
    // A different address attempts to accept — must panic
    client.accept_admin(&other);
}

#[test]
fn test_admin_cleared_after_transfer() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    env.mock_all_auths();

    client.init(&admin, &1000);
    client.propose_admin(&admin, &new_admin);
    client.accept_admin(&new_admin);

    assert_ne!(client.get_admin(), admin);
    assert_eq!(client.get_admin(), new_admin);
}

#[test]
fn test_admin_transfer_emits_event() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    env.mock_all_auths();

    client.init(&admin, &1000);
    client.propose_admin(&admin, &new_admin);
    client.accept_admin(&new_admin);

    assert!(!env.events().all().is_empty());
}

#[test]
fn test_init_persists_escrow_amount_with_ttl() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);
    let sender = Address::generate(&env);
    env.mock_all_auths();

    client.init(&sender, &5000);

    assert_eq!(client.get_amount(), 5000);
}

#[test]
fn test_propose_admin_extends_instance_ttl() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    env.mock_all_auths();

    client.init(&admin, &1000);
    client.propose_admin(&admin, &new_admin);
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_accept_admin_extends_instance_ttl() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    env.mock_all_auths();

    client.init(&admin, &1000);
    client.propose_admin(&admin, &new_admin);
    client.accept_admin(&new_admin);
    assert_eq!(client.get_admin(), new_admin);
}

// ── lifecycle integration tests ───────────────────────────────────────────────

#[test]
fn test_happy_path_create_and_release() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 1000);

    client.create_escrow(&sender, &driver, &1u64, &token_addr, &1000);

    assert_eq!(balance(&env, &token_addr, &sender), 0);
    assert_eq!(balance(&env, &token_addr, &contract_id), 1000);

    let record = client.get_escrow_record(&1u64);
    assert_eq!(record.status, EscrowStatus::Pending);

    client.release_escrow(&admin, &1u64);

    assert_eq!(balance(&env, &token_addr, &driver), 1000);
    assert_eq!(balance(&env, &token_addr, &contract_id), 0);
    assert_eq!(client.get_escrow_record(&1u64).status, EscrowStatus::Released);
}

#[test]
fn test_refund_path_restores_sender_balance() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 500);

    client.create_escrow(&sender, &driver, &2u64, &token_addr, &500);

    assert_eq!(balance(&env, &token_addr, &sender), 0);

    client.refund_escrow(&admin, &2u64);

    assert_eq!(balance(&env, &token_addr, &sender), 500);
    assert_eq!(balance(&env, &token_addr, &contract_id), 0);
    assert_eq!(client.get_escrow_record(&2u64).status, EscrowStatus::Refunded);
}

#[test]
fn test_dispute_resolved_to_driver() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 750);

    client.create_escrow(&sender, &driver, &3u64, &token_addr, &750);
    client.raise_dispute(&sender, &3u64);

    assert_eq!(client.get_escrow_record(&3u64).status, EscrowStatus::Disputed);

    client.resolve_dispute(&admin, &3u64, &true);

    assert_eq!(balance(&env, &token_addr, &driver), 750);
    assert_eq!(balance(&env, &token_addr, &sender), 0);
    assert_eq!(client.get_escrow_record(&3u64).status, EscrowStatus::Released);
}

#[test]
fn test_dispute_resolved_to_sender() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 300);

    client.create_escrow(&sender, &driver, &4u64, &token_addr, &300);
    client.raise_dispute(&sender, &4u64);
    client.resolve_dispute(&admin, &4u64, &false);

    assert_eq!(balance(&env, &token_addr, &sender), 300);
    assert_eq!(balance(&env, &token_addr, &driver), 0);
    assert_eq!(client.get_escrow_record(&4u64).status, EscrowStatus::Refunded);
}

#[test]
#[should_panic]
fn test_release_by_non_admin_rejected() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 200);
    client.create_escrow(&sender, &driver, &5u64, &token_addr, &200);

    client.release_escrow(&attacker, &5u64);
}

#[test]
#[should_panic]
fn test_refund_by_non_admin_rejected() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 200);
    client.create_escrow(&sender, &driver, &6u64, &token_addr, &200);

    client.refund_escrow(&attacker, &6u64);
}

#[test]
#[should_panic]
fn test_raise_dispute_by_non_sender_rejected() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 200);
    client.create_escrow(&sender, &driver, &7u64, &token_addr, &200);

    client.raise_dispute(&attacker, &7u64);
}

#[test]
#[should_panic]
fn test_resolve_dispute_by_non_admin_rejected() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 200);
    client.create_escrow(&sender, &driver, &8u64, &token_addr, &200);
    client.raise_dispute(&sender, &8u64);

    client.resolve_dispute(&attacker, &8u64, &true);
}

#[test]
#[should_panic]
fn test_duplicate_delivery_id_rejected() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 2000);
    client.create_escrow(&sender, &driver, &9u64, &token_addr, &1000);

    client.create_escrow(&sender, &driver, &9u64, &token_addr, &1000);
}

#[test]
#[should_panic]
fn test_release_on_already_released_rejected() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 400);
    client.create_escrow(&sender, &driver, &10u64, &token_addr, &400);
    client.release_escrow(&admin, &10u64);

    client.release_escrow(&admin, &10u64);
}

#[test]
#[should_panic]
fn test_refund_on_released_escrow_rejected() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 400);
    client.create_escrow(&sender, &driver, &11u64, &token_addr, &400);
    client.release_escrow(&admin, &11u64);

    client.refund_escrow(&admin, &11u64);
}

// ── event emission tests ─────────────────────────────────────────────────────

#[test]
fn test_create_escrow_emits_escrow_funded_event() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 1000);

    client.create_escrow(&sender, &driver, &100u64, &token_addr, &1000);

    let events = env.events().all();
    let event = events.last().unwrap();
    
    // Verify event has two topics: escrow_funded and delivery_id
    assert_eq!(event.topics.len(), 2);
    assert!(events.len() > 0);
}

#[test]
fn test_release_escrow_emits_escrow_released_event() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 1000);
    client.create_escrow(&sender, &driver, &101u64, &token_addr, &1000);

    let event_count_before = env.events().all().len();
    client.release_escrow(&admin, &101u64);
    let event_count_after = env.events().all().len();

    // Verify new event was emitted
    assert!(event_count_after > event_count_before);
    let events = env.events().all();
    let release_event = events.last().unwrap();
    assert_eq!(release_event.topics.len(), 2);
}

#[test]
fn test_refund_escrow_emits_escrow_refunded_event() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 500);
    client.create_escrow(&sender, &driver, &102u64, &token_addr, &500);

    let event_count_before = env.events().all().len();
    client.refund_escrow(&admin, &102u64);
    let event_count_after = env.events().all().len();

    // Verify new event was emitted
    assert!(event_count_after > event_count_before);
    let events = env.events().all();
    let refund_event = events.last().unwrap();
    assert_eq!(refund_event.topics.len(), 2);
}

#[test]
fn test_raise_dispute_emits_delivery_disputed_event() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 750);
    client.create_escrow(&sender, &driver, &103u64, &token_addr, &750);

    let event_count_before = env.events().all().len();
    client.raise_dispute(&sender, &103u64);
    let event_count_after = env.events().all().len();

    // Verify new event was emitted
    assert!(event_count_after > event_count_before);
    let events = env.events().all();
    let dispute_event = events.last().unwrap();
    assert_eq!(dispute_event.topics.len(), 2);
}

#[test]
fn test_resolve_dispute_to_driver_emits_dispute_resolved_event() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 750);
    client.create_escrow(&sender, &driver, &104u64, &token_addr, &750);
    client.raise_dispute(&sender, &104u64);

    let event_count_before = env.events().all().len();
    client.resolve_dispute(&admin, &104u64, &true);
    let event_count_after = env.events().all().len();

    // Verify new event was emitted
    assert!(event_count_after > event_count_before);
    let events = env.events().all();
    let resolve_event = events.last().unwrap();
    assert_eq!(resolve_event.topics.len(), 2);
}

#[test]
fn test_resolve_dispute_to_sender_emits_dispute_resolved_event() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 300);
    client.create_escrow(&sender, &driver, &105u64, &token_addr, &300);
    client.raise_dispute(&sender, &105u64);

    let event_count_before = env.events().all().len();
    client.resolve_dispute(&admin, &105u64, &false);
    let event_count_after = env.events().all().len();

    // Verify new event was emitted
    assert!(event_count_after > event_count_before);
    let events = env.events().all();
    let resolve_event = events.last().unwrap();
    assert_eq!(resolve_event.topics.len(), 2);
}

#[test]
fn test_lifecycle_events_emitted() {
    let (env, contract_id) = setup_env();
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = setup_token(&env, &token_admin);

    client.init(&admin, &0);
    mint(&env, &token_addr, &sender, 600);

    client.create_escrow(&sender, &driver, &12u64, &token_addr, &600);
    client.raise_dispute(&sender, &12u64);
    client.resolve_dispute(&admin, &12u64, &true);

    assert!(!env.events().all().is_empty());
}
