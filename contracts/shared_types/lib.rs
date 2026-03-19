#![no_std]

use soroban_sdk::{contracttype, String};

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
