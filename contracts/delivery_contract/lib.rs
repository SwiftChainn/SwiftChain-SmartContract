#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, Symbol,
};

pub type DeliveryId = u64;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    Active,
    Completed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryRecord {
    pub driver: Option<Address>,
    pub status: DeliveryStatus,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Delivery(DeliveryId),
    Admin,
}

#[contract]
pub struct DeliveryContract;

#[contractimpl]
impl DeliveryContract {
    pub fn init_admin(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn create_delivery(env: Env, delivery_id: DeliveryId) {
        let record = DeliveryRecord {
            driver: None,
            status: DeliveryStatus::Pending,
        };
        env.storage().persistent().set(&DataKey::Delivery(delivery_id), &record);
    }

    pub fn assign_driver(
        env: Env,
        caller: Address,
        delivery_id: DeliveryId,
        driver: Address,
    ) {
        caller.require_auth();

        let is_admin = Self::is_admin(&env, &caller);
        let is_self_assignment = caller == driver;

        if !is_admin && !is_self_assignment {
            panic!("NotAuthorized");
        }

        let key = DataKey::Delivery(delivery_id);
        let mut delivery: DeliveryRecord = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("DeliveryNotFound"));

        if delivery.status != DeliveryStatus::Pending {
            panic!("InvalidState");
        }

        delivery.driver = Some(driver.clone());
        delivery.status = DeliveryStatus::Active;

        env.storage().persistent().set(&key, &delivery);

        // Extend TTL: ~30 days
        env.storage().persistent().extend_ttl(&key, 518400, 518400);

        env.events().publish(
            (Symbol::new(&env, "driver_assigned"),),
            (delivery_id, driver),
        );
    }

    fn is_admin(env: &Env, caller: &Address) -> bool {
        if let Some(admin) = env.storage().instance().get::<_, Address>(&DataKey::Admin) {
            admin == *caller
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test;

