#![no_std]

use soroban_sdk::{contract, contractimpl, Env, Symbol, Address};
use shared_types::DeliveryStatus;

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize the escrow with an amount
    pub fn init(env: Env, sender: Address, amount: i128) {
        sender.require_auth();
        env.storage().instance().set(&Symbol::new(&env, "amount"), &amount);
    }

    /// Retrieve the delivery status for the escrow (Sample cross-contract integration)
    pub fn get_status(_env: Env) -> DeliveryStatus {
        DeliveryStatus::Created
    }
}

#[cfg(test)]
mod test;
