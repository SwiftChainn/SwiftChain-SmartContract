#![no_std]

use soroban_sdk::{contracttype, String};

// Event topic constants for on-chain event tracking
pub mod events {
    use soroban_sdk::{Env, Symbol};

    pub fn escrow_funded(env: &Env) -> Symbol {
        Symbol::new(env, "escrow_funded")
    }

    pub fn escrow_released(env: &Env) -> Symbol {
        Symbol::new(env, "escrow_released")
    }

    pub fn escrow_refunded(env: &Env) -> Symbol {
        Symbol::new(env, "escrow_refunded")
    }

    pub fn delivery_disputed(env: &Env) -> Symbol {
        Symbol::new(env, "delivery_disputed")
    }

    pub fn dispute_resolved(env: &Env) -> Symbol {
        Symbol::new(env, "dispute_resolved")
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryStatus {
    Created,
    InTransit,
    Delivered,
    Disputed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryDetails {
    pub id: u64,
    pub driver: String,
    pub status: DeliveryStatus,
}
