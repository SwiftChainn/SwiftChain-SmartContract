#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

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

    // old admin is no longer the admin
    assert_ne!(client.get_admin(), admin);
    // new admin is set
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

    // At least the AdminTransferred event was published during accept_admin
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

    // Verifies amount was written to persistent storage and TTL extension did not panic
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
    // Verifies instance TTL extension in propose_admin does not panic and state is correct
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
    // Verifies instance TTL extension in accept_admin does not panic and admin is updated
    client.accept_admin(&new_admin);
    assert_eq!(client.get_admin(), new_admin);
}
