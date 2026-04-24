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
    client.create_delivery(&delivery_id);

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
    client.create_delivery(&delivery_id);

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
    let (_env, client, _, driver, unauthorized) = setup_test();

    let delivery_id = 3;
    client.create_delivery(&delivery_id);

    client.assign_driver(&unauthorized, &delivery_id, &driver);
}

#[test]
#[should_panic(expected = "InvalidState")]
fn test_assignment_when_status_not_pending() {
    let (env, client, admin, driver, _) = setup_test();

    let delivery_id = 4;
    client.create_delivery(&delivery_id);

    // First assignment changes status to Active
    client.assign_driver(&admin, &delivery_id, &driver);

    // Second assignment should fail because status is Active
    let another_driver = Address::generate(&env);
    client.assign_driver(&admin, &delivery_id, &another_driver);
}



