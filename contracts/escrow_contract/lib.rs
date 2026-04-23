#![no_std]

use soroban_sdk::{contract, contractimpl, token, Env, Symbol, Address};
use shared_types::{DeliveryStatus, events};

mod constants {
    // Ledger closes ~every 5 seconds; 17,280 ledgers ≈ 1 day.
    // Trigger re-extension when fewer than ~30 days of ledgers remain.
    pub const ESCROW_TTL_THRESHOLD: u32 = 518_400;
    // Extend to ~90 days to cover the full delivery lifecycle including disputes.
    pub const ESCROW_TTL_EXTEND_TO: u32 = 1_555_200;
}

#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Pending,
    Released,
    Refunded,
    Disputed,
}

#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowRecord {
    pub sender: Address,
    pub driver: Address,
    pub token: Address,
    pub amount: i128,
    pub status: EscrowStatus,
}

#[soroban_sdk::contracttype]
pub enum DataKey {
    Escrow(u64),
}

fn load_escrow(env: &Env, delivery_id: u64) -> EscrowRecord {
    env.storage()
        .persistent()
        .get(&DataKey::Escrow(delivery_id))
        .unwrap()
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

fn require_admin(env: &Env, caller: &Address) {
    let admin: Address = env
        .storage()
        .instance()
        .get(&Symbol::new(env, "admin"))
        .unwrap();
    if *caller != admin {
        panic!("caller is not the admin");
    }
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn init(env: Env, sender: Address, amount: i128) {
        sender.require_auth();
        let amount_key = Symbol::new(&env, "amount");
        env.storage().persistent().set(&amount_key, &amount);
        env.storage().persistent().extend_ttl(
            &amount_key,
            constants::ESCROW_TTL_THRESHOLD,
            constants::ESCROW_TTL_EXTEND_TO,
        );
        env.storage().instance().set(&Symbol::new(&env, "admin"), &sender);
        env.storage().instance().extend_ttl(
            constants::ESCROW_TTL_THRESHOLD,
            constants::ESCROW_TTL_EXTEND_TO,
        );
    }

    pub fn get_status(_env: Env) -> DeliveryStatus {
        DeliveryStatus::Created
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&Symbol::new(&env, "admin"))
            .unwrap()
    }

    pub fn get_amount(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&Symbol::new(&env, "amount"))
            .unwrap()
    }

    pub fn propose_admin(env: Env, current_admin: Address, new_admin: Address) {
        current_admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "admin"))
            .unwrap();
        if stored_admin != current_admin {
            panic!("caller is not the admin");
        }
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "pending_admin"), &new_admin);
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
            .get(&Symbol::new(&env, "pending_admin"))
            .unwrap();
        if pending != new_admin {
            panic!("caller is not the pending admin");
        }
        let old_admin: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "admin"))
            .unwrap();
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "admin"), &new_admin);
        env.storage()
            .instance()
            .remove(&Symbol::new(&env, "pending_admin"));
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
        env.events().publish(
            (events::escrow_funded(), delivery_id),
            (sender, amount),
        );
    }

    pub fn release_escrow(env: Env, caller: Address, delivery_id: u64) {
        caller.require_auth();
        require_admin(&env, &caller);
        let mut record = load_escrow(&env, delivery_id);
        if record.status != EscrowStatus::Pending {
            panic!("escrow is not in pending state");
        }
        token::Client::new(&env, &record.token).transfer(
            &env.current_contract_address(),
            &record.driver,
            &record.amount,
        );
        record.status = EscrowStatus::Released;
        save_escrow(&env, delivery_id, &record);
        env.events().publish(
            (events::escrow_released(), delivery_id),
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
        token::Client::new(&env, &record.token).transfer(
            &env.current_contract_address(),
            &record.sender,
            &record.amount,
        );
        record.status = EscrowStatus::Refunded;
        save_escrow(&env, delivery_id, &record);
        env.events().publish(
            (events::escrow_refunded(), delivery_id),
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
            (events::delivery_disputed(), delivery_id),
            (caller, timestamp),
        );
    }

    pub fn resolve_dispute(
        env: Env,
        caller: Address,
        delivery_id: u64,
        release_to_driver: bool,
    ) {
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
            (events::dispute_resolved(), delivery_id),
            (release_to_driver, caller),
        );
    }

    pub fn get_escrow_record(env: Env, delivery_id: u64) -> EscrowRecord {
        load_escrow(&env, delivery_id)
    }
}

#[cfg(test)]
mod test;
