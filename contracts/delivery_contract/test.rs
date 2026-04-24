#![cfg(test)]
 
use super::*;
use soroban_sdk::{testutils::{Address as _, Events}, Address, Env, Symbol, TryFromVal};

fn setup_test() -> (Env, DeliveryContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(DeliveryContract, ());
    let client = DeliveryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let driver = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    client.init_admin(&admin);

    (env, client, admin, driver, unauthorized)
}

#[test]
fn test_successful_assignment_by_admin() {
    let (env, client, admin, driver, _) = setup_test();

    let delivery_id = 1;
    let sender = Address::generate(&env);
    client.create_delivery(&sender, &delivery_id);

    client.assign_driver(&admin, &delivery_id, &driver);

    // Verify events
    let events = env.events().all();
    let last_event = events.last().unwrap();
    
    assert_eq!(
        last_event.0, // contract_id
        client.address.clone()
    );

    let topic0: Symbol = Symbol::try_from_val(&env, &last_event.1.get(0).unwrap()).unwrap();
    assert_eq!(topic0, Symbol::new(&env, "driver_assigned"));

    let data: (DeliveryId, Address) = <(DeliveryId, Address)>::try_from_val(&env, &last_event.2).unwrap();
    assert_eq!(data, (delivery_id, driver.clone()));

    let delivery: DeliveryRecord = env
        .as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&DataKey::Delivery(delivery_id))
                .unwrap()
        });

    assert_eq!(delivery.driver, Some(driver.clone()));
    assert_eq!(delivery.status, DeliveryStatus::Active);
}

#[test]
fn test_successful_self_assignment_by_driver() {
    let (env, client, _, driver, _) = setup_test();

    let delivery_id = 2;
    let sender = Address::generate(&env);
    client.create_delivery(&sender, &delivery_id);

    client.assign_driver(&driver, &delivery_id, &driver);

    let delivery: DeliveryRecord = env
        .as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&DataKey::Delivery(delivery_id))
                .unwrap()
        });

    assert_eq!(delivery.driver, Some(driver));
    assert_eq!(delivery.status, DeliveryStatus::Active);
}

#[test]
#[should_panic(expected = "NotAuthorized")]
fn test_unauthorized_caller_rejected() {
    let (env, client, _, driver, unauthorized) = setup_test();

    let delivery_id = 3;
    let sender = Address::generate(&env);
    client.create_delivery(&sender, &delivery_id);

    client.assign_driver(&unauthorized, &delivery_id, &driver);
}

#[test]
#[should_panic(expected = "InvalidState")]
fn test_assignment_when_status_not_pending() {
    let (env, client, admin, driver, _) = setup_test();

    let delivery_id = 4;
    let sender = Address::generate(&env);
    client.create_delivery(&sender, &delivery_id);

    // First assignment changes status to Active
    client.assign_driver(&admin, &delivery_id, &driver);

    // Second assignment should fail because status is Active
    let another_driver = Address::generate(&env);
    client.assign_driver(&admin, &delivery_id, &another_driver);
}

// --- Escrow Mock ---
#[contract]
pub struct MockEscrow;

#[contractimpl]
impl MockEscrow {
    pub fn refund_escrow(_env: Env, delivery_id: DeliveryId) {
        // We can simulate failure if delivery_id is a specific value
        if delivery_id == 999 {
            panic!("Escrow failure simulated");
        }
    }
}

#[test]
fn test_cancel_delivery_pending() {
    let (env, client, admin, _, _) = setup_test();
    let escrow_id = env.register(MockEscrow, ());
    client.set_escrow_contract(&admin, &escrow_id);

    let delivery_id = 10;
    let sender = Address::generate(&env);
    client.create_delivery(&sender, &delivery_id);

    client.cancel_delivery(&sender, &delivery_id);

    let delivery: DeliveryRecord = env
        .as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&DataKey::Delivery(delivery_id))
                .unwrap()
        });

    assert_eq!(delivery.status, DeliveryStatus::Cancelled);

    let events = env.events().all();
    let last_event = events.last().unwrap();
    let topic0: Symbol = Symbol::try_from_val(&env, &last_event.1.get(0).unwrap()).unwrap();
    assert_eq!(topic0, Symbol::new(&env, "delivery_cancelled"));
}

#[test]
fn test_cancel_delivery_active() {
    let (env, client, admin, driver, _) = setup_test();
    let escrow_id = env.register(MockEscrow, ());
    client.set_escrow_contract(&admin, &escrow_id);

    let delivery_id = 11;
    let sender = Address::generate(&env);
    client.create_delivery(&sender, &delivery_id);
    client.assign_driver(&admin, &delivery_id, &driver);

    client.cancel_delivery(&sender, &delivery_id);

    let delivery: DeliveryRecord = env
        .as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&DataKey::Delivery(delivery_id))
                .unwrap()
        });

    assert_eq!(delivery.status, DeliveryStatus::Cancelled);
}

#[test]
#[should_panic(expected = "NotAuthorized")]
fn test_cancel_delivery_unauthorized() {
    let (env, client, admin, _, unauthorized) = setup_test();
    let escrow_id = env.register(MockEscrow, ());
    client.set_escrow_contract(&admin, &escrow_id);

    let delivery_id = 12;
    let sender = Address::generate(&env);
    client.create_delivery(&sender, &delivery_id);

    client.cancel_delivery(&unauthorized, &delivery_id);
}

#[test]
#[should_panic(expected = "InvalidState")]
fn test_cancel_delivery_invalid_state() {
    let (env, client, admin, _, _) = setup_test();
    let escrow_id = env.register(MockEscrow, ());
    client.set_escrow_contract(&admin, &escrow_id);

    let delivery_id = 13;
    let sender = Address::generate(&env);
    client.create_delivery(&sender, &delivery_id);

    client.cancel_delivery(&sender, &delivery_id); // Now Cancelled

    // Try cancelling again -> should fail with InvalidState
    client.cancel_delivery(&sender, &delivery_id);
}

#[test]
#[should_panic(expected = "Escrow failure simulated")]
fn test_cancel_delivery_escrow_failure() {
    let (env, client, admin, _, _) = setup_test();
    let escrow_id = env.register(MockEscrow, ());
    client.set_escrow_contract(&admin, &escrow_id);

    let delivery_id = 999; // trigger failure in mock
    let sender = Address::generate(&env);
    client.create_delivery(&sender, &delivery_id);

    client.cancel_delivery(&sender, &delivery_id);
}



