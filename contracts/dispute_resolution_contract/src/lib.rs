#![no_std]

// dispute_resolution_contract — raise_dispute implementation (WIP)
// Tracks dispute cases linked to delivery_id, validates caller is sender or receiver,
// ensures delivery is in an active state before escalating.

use soroban_sdk::{contract, contractimpl, Address, Env};

pub type DeliveryId = u64;

#[contract]
pub struct DisputeResolutionContract;

#[contractimpl]
impl DisputeResolutionContract {
    // raise_dispute: implementation in progress
    pub fn raise_dispute(_env: Env, _caller: Address, _delivery_id: DeliveryId) {
        todo!()
    }
}
