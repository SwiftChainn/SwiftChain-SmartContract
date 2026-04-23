#![no_std]

use soroban_sdk::{contract, contractimpl, Env, Symbol, Address};
use shared_types::DeliveryStatus;

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn init(env: Env, sender: Address, amount: i128) {
        sender.require_auth();
        env.storage().instance().set(&Symbol::new(&env, "amount"), &amount);
        env.storage().instance().set(&Symbol::new(&env, "admin"), &sender);
    }

    pub fn get_status(_env: Env) -> DeliveryStatus {
        DeliveryStatus::Created
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance()
            .get(&Symbol::new(&env, "admin"))
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
        env.events().publish(
            (Symbol::new(&env, "AdminTransferred"),),
            (old_admin, new_admin),
        );
    }
}

#[cfg(test)]
mod test;
