#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

#[test]
fn test_init_and_get_status() {
    let env = Env::default();
    
    // Register the contract
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);

    // Generate a mock sender address
    let sender = Address::generate(&env);
    
    // Mock authentication for the sender
    env.mock_all_auths();

    // Call the init function
    client.init(&sender, &1000);

    // Call get_status and verify the result
    let status = client.get_status();
    assert_eq!(status, DeliveryStatus::Created);
}
