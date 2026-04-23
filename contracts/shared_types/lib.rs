#![no_std]

use soroban_sdk::{contracttype, String, symbol_short};

// Event topic constants for on-chain event tracking
pub mod events {
    use soroban_sdk::Symbol;

    pub fn escrow_funded() -> Symbol {
        symbol_short!("escrow_funded")
    }

    pub fn escrow_released() -> Symbol {
        symbol_short!("escrow_released")
    }

    pub fn escrow_refunded() -> Symbol {
        symbol_short!("escrow_refunded")
    }

    pub fn delivery_disputed() -> Symbol {
        symbol_short!("delivery_disputed")
    }

    pub fn dispute_resolved() -> Symbol {
        symbol_short!("dispute_resolved")
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
