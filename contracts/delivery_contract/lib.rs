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
    InTransit,
    Delivered,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryMetadata {
    pub recipient: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryRecord {
    pub sender: Address,
    pub driver: Option<Address>,
    pub status: DeliveryStatus,
    pub metadata: DeliveryMetadata,
    pub delivered_at: Option<u64>,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Delivery(DeliveryId),
    Admin,
    EscrowContract,
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

    pub fn set_escrow_contract(env: Env, admin: Address, escrow: Address) {
        admin.require_auth();
        if !Self::is_admin(&env, &admin) {
            panic!("NotAuthorized");
        }
        env.storage().instance().set(&DataKey::EscrowContract, &escrow);
    }

    pub fn create_delivery(env: Env, sender: Address, recipient: Address, delivery_id: DeliveryId) {
        sender.require_auth();
        let record = DeliveryRecord {
            sender: sender.clone(),
            driver: None,
            status: DeliveryStatus::Pending,
            metadata: DeliveryMetadata { recipient },
            delivered_at: None,
        };
        env.storage().persistent().set(&DataKey::Delivery(delivery_id), &record);
    }

    pub fn cancel_delivery(
        env: Env,
        sender: Address,
        delivery_id: DeliveryId,
    ) {
        sender.require_auth();

        let key = DataKey::Delivery(delivery_id);
        let mut delivery: DeliveryRecord = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("DeliveryNotFound"));

        if delivery.sender != sender {
            panic!("NotAuthorized");
        }

        if delivery.status != DeliveryStatus::Pending && delivery.status != DeliveryStatus::Active {
            panic!("InvalidState");
        }

        let escrow_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::EscrowContract)
            .unwrap_or_else(|| panic!("EscrowNotConfigured"));

        use soroban_sdk::IntoVal;
        let _: () = env.invoke_contract(
            &escrow_address,
            &soroban_sdk::Symbol::new(&env, "refund_escrow"),
            soroban_sdk::vec![&env, delivery_id.into_val(&env)],
        );

        delivery.status = DeliveryStatus::Cancelled;
        env.storage().persistent().set(&key, &delivery);
        env.storage().persistent().extend_ttl(&key, 518400, 518400);

        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "delivery_cancelled"),),
            (delivery_id, sender),
        );
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

    pub fn confirm_delivery(
        env: Env,
        recipient: Address,
        delivery_id: DeliveryId,
    ) {
        recipient.require_auth();

        let key = DataKey::Delivery(delivery_id);
        let mut delivery: DeliveryRecord = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("DeliveryNotFound"));

        if recipient != delivery.metadata.recipient {
            panic!("NotAuthorized");
        }

        if delivery.status != DeliveryStatus::InTransit {
            panic!("InvalidState");
        }

        let escrow_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::EscrowContract)
            .unwrap_or_else(|| panic!("EscrowNotConfigured"));

        use soroban_sdk::IntoVal;
        let _: () = env.invoke_contract(
            &escrow_address,
            &soroban_sdk::Symbol::new(&env, "release_escrow"),
            soroban_sdk::vec![&env, delivery_id.into_val(&env)],
        );

        delivery.status = DeliveryStatus::Delivered;
        delivery.delivered_at = Some(env.ledger().timestamp());

        env.storage().persistent().set(&key, &delivery);
        env.storage().persistent().extend_ttl(&key, 518400, 518400);

        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "delivery_confirmed"),),
            (delivery_id, recipient),
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

