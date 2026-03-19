#![cfg(test)]

use super::*;
use soroban_sdk::{Env, String};

#[test]
fn test_create_delivery() {
    let env = Env::default();
    
    // Register the contract
    let contract_id = env.register_contract(None, DeliveryContract);
    let client = DeliveryContractClient::new(&env, &contract_id);

    // Create mock parameters
    let driver_name = String::from_str(&env, "DriverBob");
    
    // Call create_delivery
    let delivery = client.create_delivery(&1, &driver_name);

    // Assert the properties of the created delivery
    assert_eq!(delivery.id, 1);
    assert_eq!(delivery.driver, driver_name);
    assert_eq!(delivery.status, DeliveryStatus::Created);
}
