#![no_std]

use soroban_sdk::{contract, contractimpl, Env, String};
use shared_types::{DeliveryDetails, DeliveryStatus};

#[contract]
pub struct DeliveryContract;

#[contractimpl]
impl DeliveryContract {
    /// Create a new delivery record (Sample cross-contract integration using shared_types)
    pub fn create_delivery(_env: Env, id: u64, driver: String) -> DeliveryDetails {
        DeliveryDetails {
            id,
            driver,
            status: DeliveryStatus::Created,
        }
    }
}

#[cfg(test)]
mod test;
