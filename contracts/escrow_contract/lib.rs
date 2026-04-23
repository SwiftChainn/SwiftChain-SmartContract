#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, contracterror, Env, Symbol, Address, panic_with_error};
use shared_types::DeliveryStatus;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    PlatformFeeBps,
    Amount,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    InvalidState = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeUpdated {
    pub old_fee: u32,
    pub new_fee: u32,
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize the escrow with an admin and amount
    pub fn init(env: Env, admin: Address, amount: i128) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Amount, &amount);
        env.storage().instance().set(&DataKey::PlatformFeeBps, &0u32);
    }

    /// Update the platform fee in basis points (max 1000 = 10%)
    pub fn update_platform_fee(env: Env, admin: Address, new_fee_bps: u32) {
        // 1. Verify against stored admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Not initialized");
        if admin != stored_admin {
            panic!("Unauthorized");
        }

        // 2. Require authentication
        admin.require_auth();

        // 3. Validate fee <= 1000 bps
        if new_fee_bps > 1000 {
            panic_with_error!(&env, EscrowError::InvalidState);
        }

        // 4. Update storage and emit event
        let old_fee: u32 = env.storage().instance().get(&DataKey::PlatformFeeBps).unwrap_or(0);
        env.storage().instance().set(&DataKey::PlatformFeeBps, &new_fee_bps);

        env.events().publish(
            (Symbol::new(&env, "FeeUpdated"),),
            FeeUpdated { old_fee, new_fee: new_fee_bps }
        );
    }

    /// Get current platform fee in basis points
    pub fn get_platform_fee(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::PlatformFeeBps).unwrap_or(0)
    }

    /// Retrieve the delivery status for the escrow
    pub fn get_status(_env: Env) -> DeliveryStatus {
        DeliveryStatus::Created
    }
}

#[cfg(test)]
mod test;
