#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Events}, Env, vec, IntoVal};

#[test]
fn test_init_and_get_status() {
    let env = Env::default();
    
    // Register the contract
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);

    // Generate a mock admin address
    let admin = Address::generate(&env);
    
    // Mock authentication
    env.mock_all_auths();

    // Call the init function
    client.init(&admin, &1000);

    // Call get_status and verify the result
    let status = client.get_status();
    assert_eq!(status, DeliveryStatus::Created);

    // Verify initial fee is 0
    assert_eq!(client.get_platform_fee(), 0);
}

#[test]
fn test_update_platform_fee_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.init(&admin, &1000);

    // Update fee to 5% (500 bps)
    client.update_platform_fee(&admin, &500);

    assert_eq!(client.get_platform_fee(), 500);

    // Verify event emission
    let events = env.events().all();
    panic!("Events: {:?}", events);
    let last_event = events.last().unwrap();
    
    assert_eq!(last_event.0, contract_id);
    
    // Check topics
    let topics = last_event.1;
    assert_eq!(topics.len(), 1);
    let topic_sym: Symbol = topics.get(0).unwrap().into_val(&env);
    assert_eq!(topic_sym, Symbol::new(&env, "FeeUpdated"));
    
    // Check value
    let event_value: FeeUpdated = last_event.2.into_val(&env);
    assert_eq!(event_value, FeeUpdated { old_fee: 1000, new_fee: 500 });
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_update_platform_fee_unauthorized() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let malicious_user = Address::generate(&env);
    env.mock_all_auths();

    client.init(&admin, &1000);

    // Malicious user tries to update fee
    client.update_platform_fee(&malicious_user, &500);
}

#[test]
fn test_update_platform_fee_invalid_value() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.init(&admin, &1000);

    // Try to update fee to 11% (1100 bps) - should fail with InvalidState
    let result = client.try_update_platform_fee(&admin, &1100);
    
    match result {
        Err(Ok(err)) => assert_eq!(err, EscrowError::InvalidState.into()),
        _ => panic!("Expected EscrowError::InvalidState, got {:?}", result),
    }
}
