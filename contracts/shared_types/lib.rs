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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CargoCategory {
    Documents,
    Electronics,
    Perishables,
    Clothing,
    General,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoDescriptor {
    pub weight_grams: u32,
    pub category: CargoCategory,
    pub fragile: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryMetadata {
    pub delivery_id: u64,
    pub origin: String,
    pub destination: String,
    pub cargo_description: CargoDescriptor,
    pub created_at: u64,
    pub estimated_delivery: u64,
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Env, String};

    #[test]
    fn test_cargo_descriptor() {
        let _env = Env::default();
        let desc = CargoDescriptor {
            weight_grams: 500,
            category: CargoCategory::Electronics,
            fragile: true,
        };
        assert_eq!(desc.weight_grams, 500);
        assert_eq!(desc.fragile, true);
        assert_eq!(desc.category, CargoCategory::Electronics);
    }

    #[test]
    fn test_delivery_metadata() {
        let env = Env::default();
        let cargo = CargoDescriptor {
            weight_grams: 1000,
            category: CargoCategory::General,
            fragile: false,
        };
        let metadata = DeliveryMetadata {
            delivery_id: 1,
            origin: String::from_str(&env, "Location A"),
            destination: String::from_str(&env, "Location B"),
            cargo_description: cargo,
            created_at: 1000000,
            estimated_delivery: 2000000,
        };
        assert_eq!(metadata.delivery_id, 1);
        assert_eq!(metadata.created_at, 1000000);
        assert_eq!(metadata.cargo_description.weight_grams, 1000);
    }
}
