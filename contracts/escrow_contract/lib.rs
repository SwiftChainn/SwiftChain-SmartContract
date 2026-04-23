#![no_std]

use soroban_sdk::{contract, contractimpl, Env, Symbol, Address};
use shared_types::DeliveryStatus;

mod constants {
    // Ledger closes ~every 5 seconds; 17,280 ledgers ≈ 1 day.
    // Trigger re-extension when fewer than ~30 days of ledgers remain.
    pub const ESCROW_TTL_THRESHOLD: u32 = 518_400;
    // Extend to ~90 days to cover the full delivery lifecycle including disputes.
    pub const ESCROW_TTL_EXTEND_TO: u32 = 1_555_200;
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
        env.storage().instance()
            .get(&Symbol::new(&env, "admin"))
            .unwrap()
    }

    pub fn get_amount(env: Env) -> i128 {
        env.storage().persistent()
            .get(&Symbol::new(&env, "amount"))
            .unwrap()
    }

    pub fn propose_admin(env: Env, current_admin: Address, new_admin: Address) {
        current_admin.require_auth();
        let stored_admin: Address = env.storage().instance()
            .get(&Symbol::new(&env, "admin"))
            .unwrap();
        if stored_admin != current_admin {
            panic!("caller is not the admin");
        }
        env.storage().instance().set(&Symbol::new(&env, "pending_admin"), &new_admin);
        env.storage().instance().extend_ttl(
            constants::ESCROW_TTL_THRESHOLD,
            constants::ESCROW_TTL_EXTEND_TO,
        );
    }

    pub fn accept_admin(env: Env, new_admin: Address) {
        new_admin.require_auth();
        let pending: Address = env.storage().instance()
            .get(&Symbol::new(&env, "pending_admin"))
            .unwrap();
        if pending != new_admin {
            panic!("caller is not the pending admin");
        }
        let old_admin: Address = env.storage().instance()
            .get(&Symbol::new(&env, "admin"))
            .unwrap();
        env.storage().instance().set(&Symbol::new(&env, "admin"), &new_admin);
        env.storage().instance().remove(&Symbol::new(&env, "pending_admin"));
        env.storage().instance().extend_ttl(
            constants::ESCROW_TTL_THRESHOLD,
            constants::ESCROW_TTL_EXTEND_TO,
        );
        env.events().publish(
            (Symbol::new(&env, "AdminTransferred"),),
            (old_admin, new_admin),
        );
    }
}

#[cfg(test)]
mod test;
