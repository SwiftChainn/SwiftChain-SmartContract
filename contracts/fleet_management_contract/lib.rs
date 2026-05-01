#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

pub type FleetId = u64;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FleetProfile {
    pub fleet_id: FleetId,
    pub owner: Address,
    pub treasury: Address,
    pub total_active_drivers: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    FleetCounter,
    Fleet(FleetId),
}

#[contract]
pub struct FleetManagementContract;

#[contractimpl]
impl FleetManagementContract {
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("AlreadyInitialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::FleetCounter, &0u64);

        env.events().publish(
            (Symbol::new(&env, "FleetContractInitialized"),),
            admin,
        );
    }

    pub fn register_fleet(env: Env, owner: Address, treasury: Address) -> FleetId {
        owner.require_auth();

        let mut counter: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::FleetCounter)
            .unwrap_or(0);
        counter += 1;
        env.storage()
            .persistent()
            .set(&DataKey::FleetCounter, &counter);

        let fleet_id = counter;

        let profile = FleetProfile {
            fleet_id,
            owner: owner.clone(),
            treasury: treasury.clone(),
            total_active_drivers: 0,
        };

        let key = DataKey::Fleet(fleet_id);
        env.storage().persistent().set(&key, &profile);
        env.storage().persistent().extend_ttl(&key, 518400, 518400);

        env.events().publish(
            (Symbol::new(&env, "fleet_registered"),),
            (fleet_id, owner, treasury),
        );

        fleet_id
    }

    pub fn get_fleet(env: Env, fleet_id: FleetId) -> FleetProfile {
        let key = DataKey::Fleet(fleet_id);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("FleetNotFound"))
    }
}

#[cfg(test)]
mod test;
