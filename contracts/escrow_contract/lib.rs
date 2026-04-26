#![no_std]

use shared_types::{events, DeliveryStatus};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, token, Address, Env,
    Symbol,
};

mod constants {
    pub const ESCROW_TTL_THRESHOLD: u32 = 518400;
    pub const ESCROW_TTL_EXTEND_TO: u32 = 518400;
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    PendingAdmin,
    PlatformFeeBps,
    Amount,
    Escrow(u64),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    InvalidState = 1,
    DeliveryNotFound = 2,
    InsufficientFunds = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Pending,
    Released,
    Refunded,
    Disputed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowRecord {
    pub sender: Address,
    pub driver: Address,
    pub token: Address,
    pub amount: i128,
    pub status: EscrowStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeUpdated {
    pub old_fee: u32,
    pub new_fee: u32,
}

fn save_escrow(env: &Env, delivery_id: u64, record: &EscrowRecord) {
    let key = DataKey::Escrow(delivery_id);
    env.storage().persistent().set(&key, record);
    env.storage().persistent().extend_ttl(
        &key,
        constants::ESCROW_TTL_THRESHOLD,
        constants::ESCROW_TTL_EXTEND_TO,
    );
}

fn load_escrow(env: &Env, delivery_id: u64) -> EscrowRecord {
    env.storage()
        .persistent()
        .get(&DataKey::Escrow(delivery_id))
        .unwrap_or_else(|| panic_with_error!(env, EscrowError::DeliveryNotFound))
}

fn require_admin(env: &Env, caller: &Address) {
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("Not initialized");
    if *caller != stored_admin {
        panic!("Unauthorized");
    }
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn init(env: Env, admin: Address, amount: i128) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Amount, &amount);
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeeBps, &0u32);
    }

    pub fn update_platform_fee(env: Env, admin: Address, new_fee_bps: u32) {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        if admin != stored_admin {
            panic!("Unauthorized");
        }
        admin.require_auth();
        if new_fee_bps > 1000 {
            panic_with_error!(&env, EscrowError::InvalidState);
        }
        let old_fee: u32 = env
            .storage()
            .instance()
            .get(&DataKey::PlatformFeeBps)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeeBps, &new_fee_bps);
        env.events().publish(
            (Symbol::new(&env, "FeeUpdated"),),
            FeeUpdated {
                old_fee,
                new_fee: new_fee_bps,
            },
        );
    }

    pub fn get_platform_fee(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::PlatformFeeBps)
            .unwrap_or(0)
    }

    pub fn get_status(_env: Env) -> DeliveryStatus {
        DeliveryStatus::Created
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized")
    }

    pub fn get_amount(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::Amount).unwrap_or(0)
    }

    pub fn propose_admin(env: Env, current_admin: Address, new_admin: Address) {
        current_admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        if stored_admin != current_admin {
            panic!("caller is not the admin");
        }
        env.storage()
            .instance()
            .set(&DataKey::PendingAdmin, &new_admin);
        env.storage().instance().extend_ttl(
            constants::ESCROW_TTL_THRESHOLD,
            constants::ESCROW_TTL_EXTEND_TO,
        );
    }

    pub fn accept_admin(env: Env, new_admin: Address) {
        new_admin.require_auth();
        let pending: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .expect("no pending admin");
        if pending != new_admin {
            panic!("caller is not the pending admin");
        }
        let old_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        env.storage().instance().extend_ttl(
            constants::ESCROW_TTL_THRESHOLD,
            constants::ESCROW_TTL_EXTEND_TO,
        );
        env.events().publish(
            (Symbol::new(&env, "AdminTransferred"),),
            (old_admin, new_admin),
        );
    }

    // ── Escrow lifecycle ──────────────────────────────────────────────────────

    pub fn create_escrow(
        env: Env,
        sender: Address,
        driver: Address,
        delivery_id: u64,
        token: Address,
        amount: i128,
    ) {
        sender.require_auth();
        if env
            .storage()
            .persistent()
            .has(&DataKey::Escrow(delivery_id))
        {
            panic!("escrow already exists for this delivery_id");
        }
        token::Client::new(&env, &token).transfer(
            &sender,
            &env.current_contract_address(),
            &amount,
        );
        save_escrow(
            &env,
            delivery_id,
            &EscrowRecord {
                sender: sender.clone(),
                driver,
                token,
                amount,
                status: EscrowStatus::Pending,
            },
        );
        env.events()
            .publish((events::escrow_funded(&env), delivery_id), (sender, amount));
    }

    pub fn release_escrow(env: Env, caller: Address, delivery_id: u64) {
        caller.require_auth();
        require_admin(&env, &caller);
        let mut record = load_escrow(&env, delivery_id);
        if record.status != EscrowStatus::Pending {
            panic!("escrow is not in pending state");
        }
        // Balance verification guard: confirm contract holds sufficient funds before transfer
        let contract_balance =
            token::Client::new(&env, &record.token).balance(&env.current_contract_address());
        if contract_balance < record.amount {
            panic_with_error!(&env, EscrowError::InsufficientFunds);
        }
        token::Client::new(&env, &record.token).transfer(
            &env.current_contract_address(),
            &record.driver,
            &record.amount,
        );
        record.status = EscrowStatus::Released;
        save_escrow(&env, delivery_id, &record);
        env.events().publish(
            (events::escrow_released(&env), delivery_id),
            (record.driver, record.amount, 0i128),
        );
    }

    pub fn refund_escrow(env: Env, caller: Address, delivery_id: u64) {
        caller.require_auth();
        require_admin(&env, &caller);
        let mut record = load_escrow(&env, delivery_id);
        if record.status != EscrowStatus::Pending {
            panic!("escrow is not in pending state");
        }
        // Balance verification guard: confirm contract holds sufficient funds before transfer
        let contract_balance =
            token::Client::new(&env, &record.token).balance(&env.current_contract_address());
        if contract_balance < record.amount {
            panic_with_error!(&env, EscrowError::InsufficientFunds);
        }
        token::Client::new(&env, &record.token).transfer(
            &env.current_contract_address(),
            &record.sender,
            &record.amount,
        );
        record.status = EscrowStatus::Refunded;
        save_escrow(&env, delivery_id, &record);
        env.events().publish(
            (events::escrow_refunded(&env), delivery_id),
            (record.sender, record.amount),
        );
    }

    pub fn raise_dispute(env: Env, caller: Address, delivery_id: u64) {
        caller.require_auth();
        let mut record = load_escrow(&env, delivery_id);
        if caller != record.sender {
            panic!("only the escrow sender can raise a dispute");
        }
        if record.status != EscrowStatus::Pending {
            panic!("escrow is not in pending state");
        }
        record.status = EscrowStatus::Disputed;
        save_escrow(&env, delivery_id, &record);
        let timestamp = env.ledger().timestamp();
        env.events().publish(
            (events::delivery_disputed(&env), delivery_id),
            (caller, timestamp),
        );
    }

    pub fn resolve_dispute(env: Env, caller: Address, delivery_id: u64, release_to_driver: bool) {
        caller.require_auth();
        require_admin(&env, &caller);
        let mut record = load_escrow(&env, delivery_id);
        if record.status != EscrowStatus::Disputed {
            panic!("escrow is not in disputed state");
        }
        if release_to_driver {
            token::Client::new(&env, &record.token).transfer(
                &env.current_contract_address(),
                &record.driver,
                &record.amount,
            );
            record.status = EscrowStatus::Released;
        } else {
            token::Client::new(&env, &record.token).transfer(
                &env.current_contract_address(),
                &record.sender,
                &record.amount,
            );
            record.status = EscrowStatus::Refunded;
        }
        save_escrow(&env, delivery_id, &record);
        env.events().publish(
            (events::dispute_resolved(&env), delivery_id),
            (release_to_driver, caller),
        );
    }

    pub fn get_escrow(env: Env, delivery_id: u64) -> EscrowRecord {
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Escrow(delivery_id))
        {
            panic_with_error!(&env, EscrowError::DeliveryNotFound);
        }
        load_escrow(&env, delivery_id)
    }
}

#[cfg(test)]
mod test;
