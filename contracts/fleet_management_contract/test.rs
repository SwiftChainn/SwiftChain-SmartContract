extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, Symbol, TryFromVal,
};

fn setup_test() -> (Env, FleetManagementContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(FleetManagementContract, ());
    let client = FleetManagementContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    (env, client, admin)
}

#[test]
fn test_init_sets_admin_and_counter() {
    let (env, client, admin) = setup_test();

    let stored_admin: Address = env.as_contract(&client.address, || {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    });
    assert_eq!(stored_admin, admin);

    let counter: u64 = env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .get(&DataKey::FleetCounter)
            .unwrap()
    });
    assert_eq!(counter, 0);
}

#[test]
#[should_panic(expected = "AlreadyInitialized")]
fn test_init_twice_panics() {
    let (_env, client, admin) = setup_test();
    client.init(&admin);
}

#[test]
fn test_register_fleet_creates_profile_with_expected_fields() {
    let (env, client, _admin) = setup_test();

    let owner = Address::generate(&env);
    let treasury = Address::generate(&env);

    let fleet_id = client.register_fleet(&owner, &treasury);
    assert_eq!(fleet_id, 1);

    let profile = client.get_fleet(&fleet_id);
    assert_eq!(profile.fleet_id, 1);
    assert_eq!(profile.owner, owner);
    assert_eq!(profile.treasury, treasury);
    assert_eq!(profile.total_active_drivers, 0);
}

#[test]
fn test_register_fleet_increments_counter() {
    let (env, client, _admin) = setup_test();

    let owner_a = Address::generate(&env);
    let treasury_a = Address::generate(&env);
    let owner_b = Address::generate(&env);
    let treasury_b = Address::generate(&env);

    let id_a = client.register_fleet(&owner_a, &treasury_a);
    let id_b = client.register_fleet(&owner_b, &treasury_b);

    assert_eq!(id_a, 1);
    assert_eq!(id_b, 2);

    let profile_b = client.get_fleet(&id_b);
    assert_eq!(profile_b.owner, owner_b);
    assert_eq!(profile_b.treasury, treasury_b);
}

#[test]
fn test_register_fleet_emits_event() {
    let (env, client, _admin) = setup_test();

    let owner = Address::generate(&env);
    let treasury = Address::generate(&env);
    let fleet_id = client.register_fleet(&owner, &treasury);

    let events = env.events().all();
    let last_event = events.last().unwrap();

    assert_eq!(last_event.0, client.address.clone());

    let topic0: Symbol = Symbol::try_from_val(&env, &last_event.1.get(0).unwrap()).unwrap();
    assert_eq!(topic0, Symbol::new(&env, "fleet_registered"));

    let data: (FleetId, Address, Address) =
        <(FleetId, Address, Address)>::try_from_val(&env, &last_event.2).unwrap();
    assert_eq!(data, (fleet_id, owner, treasury));
}

#[test]
#[should_panic(expected = "FleetNotFound")]
fn test_get_fleet_unknown_id_panics() {
    let (_env, client, _admin) = setup_test();
    client.get_fleet(&999);
}
