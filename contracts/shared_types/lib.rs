#![no_std]

use soroban_sdk::{contracttype, Address, String};

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Locked,
    Paused,
    Released,
    Refunded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowRecord {
    pub sender: Address,
    pub recipient: Address,
    pub driver: Address,
    pub token: Address,
    pub amount: i128,
    pub status: EscrowStatus,
    pub created_at: u64,
    pub disputed_by: Option<Address>,
    pub disputed_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriverProfile {
    pub address: Address,
    pub deliveries_completed: u32,
    pub reputation_score: u32,
    pub registered_at: u64,
}

