
# Delivery Contract Storage TTL Management

## Overview

The delivery contract stores delivery records, a delivery counter, and driver profile data. Soroban persistent storage entries have a time-to-live (TTL), and entries that are not maintained can eventually expire and become unavailable.

This document describes the delivery contract's current TTL behavior, the keys that require attention, and the archival risks associated with long-lived deliveries.

Relevant implementation:

- `contracts/delivery_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `contracts/escrow_contract/lib.rs`

## 1. TTL Configuration Used by the Delivery Contract

The delivery contract currently uses:

```rust
env.storage().persistent().extend_ttl(&key, 518400, 518400);
```

The two values represent the minimum TTL threshold and the TTL extension target, expressed in Soroban ledger units.

**Important:** `518400` is a ledger-based TTL value, not a number of seconds or days. Its wall-clock duration depends on the network's ledger timing.

The delivery contract currently repeats this value directly rather than defining a dedicated TTL constant.

## 2. Delivery Contract Storage Keys

The following storage entries are relevant to delivery lifecycle persistence.

| Storage entry | Storage type | Purpose | Current TTL handling |
|---|---|---|---|
| `delivery_key(delivery_id)` | Persistent | Stores each `DeliveryRecord` | Explicitly extended on creation and lifecycle writes |
| `StorageKey::DriverProfile(address)` | Persistent | Stores a driver's delivery profile | Explicitly extended when the profile is updated during confirmation |
| `DataKey::DeliveryCounter` | Persistent | Stores the delivery identifier counter | No explicit TTL extension in the current implementation |
| `StorageKey::Admin` | Instance | Stores the delivery contract administrator | No explicit per-key persistent TTL extension |
| `DataKey::EscrowContract` | Instance | Stores the configured escrow contract address | No explicit per-key persistent TTL extension |

The `Admin` and `EscrowContract` entries are instance storage, not persistent storage entries. They should not be described as keys maintained through the delivery record's persistent-storage `extend_ttl` calls.

## 3. Delivery Record TTL

A delivery record is stored using:

```rust
let key = delivery_key(delivery_id);

env.storage().persistent().set(&key, &record);
env.storage().persistent().extend_ttl(&key, 518400, 518400);
```

The record is created in `create_delivery`.

The delivery record is also saved and its TTL extended after these lifecycle operations:

- `assign_driver`
- `mark_in_transit`
- `confirm_delivery`
- `cancel_delivery`
- `raise_dispute`

This allows an active delivery's record to receive a fresh TTL when those state-changing operations execute.

### Read behavior

The current read function is:

```rust
pub fn get_delivery(
    env: Env,
    delivery_id: DeliveryId
) -> DeliveryRecord {
    let key = delivery_key(delivery_id);

    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| panic!("DeliveryNotFound"))
}
```

**Current limitation:** `get_delivery` reads the record but does not call `extend_ttl`.

A read therefore does not refresh the delivery record's TTL in the current implementation.

For comparison, the escrow contract's `load_escrow` helper explicitly extends the escrow record TTL after loading it. The delivery contract does not currently follow that same read-extension pattern.

## 4. Driver Profile TTL

The delivery contract stores driver profile data under:

```rust
StorageKey::DriverProfile(driver_address)
```

When a delivery is confirmed and an assigned driver exists, the contract:

1. Loads the driver's profile, or constructs a fallback profile.
2. Increments `deliveries_completed`.
3. Increments `reputation_score`.
4. Saves the profile.
5. Extends the profile's persistent TTL.

The current write pattern is:

```rust
env.storage().persistent().set(&driver_key, &profile);

env.storage()
    .persistent()
    .extend_ttl(&driver_key, 518400, 518400);
```

### Read behavior

The current `get_driver_profile` function reads the profile but does not extend its TTL.

If no profile exists, it returns a fallback profile with default values. That fallback is not persisted by the read function.

Therefore, the current read function does not refresh or create a persistent profile entry.

## 5. Delivery Counter TTL

The delivery counter is stored under:

```rust
DataKey::DeliveryCounter
```

It is read and updated during `create_delivery`:

```rust
let mut counter: u64 = env
    .storage()
    .persistent()
    .get(&DataKey::DeliveryCounter)
    .unwrap_or(0);

counter += 1;

env.storage()
    .persistent()
    .set(&DataKey::DeliveryCounter, &counter);
```

**Current limitation:** The counter is persistent storage, but the current implementation does not explicitly call `extend_ttl` for this key.

The counter should be considered separately from delivery records because losing the counter may affect delivery identifier generation.

## 6. Recommended Read-and-Extend Pattern

For persistent entries that should remain available when accessed, a read helper can extend the entry's TTL after successfully loading it.

For example, the delivery record read path could follow this pattern:

```rust
pub fn get_delivery(env: Env, delivery_id: DeliveryId) -> DeliveryRecord {
    let key = delivery_key(delivery_id);

    let delivery: DeliveryRecord = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| panic!("DeliveryNotFound"));

    env.storage()
        .persistent()
        .extend_ttl(&key, 518400, 518400);

    delivery
}
```

This is a recommended pattern for future TTL maintenance. It is **not** the current implementation of `get_delivery`.

The same principle can be considered for `get_driver_profile`, provided the profile exists in persistent storage.

Any code change implementing this pattern should preserve existing storage keys, return types, and error behavior.

## 7. Archival and Long-Lived Delivery Risks

Delivery records may remain relevant after the physical delivery has been completed or cancelled.

Examples include:

- Customer support and delivery-history queries.
- Dispute investigation.
- Proof-of-delivery verification.
- Driver reputation and completed-delivery history.
- Financial reconciliation.
- Auditing and compliance requirements.

A delivery record that is not accessed or updated before its TTL expires may become unavailable from contract storage.

This is especially important for deliveries that remain in `Pending`, `Active`, `InTransit`, or `Disputed` states for an extended period.

A long-running delivery may not execute a state-changing operation frequently enough to refresh its TTL.

The same risk applies to driver profiles and the delivery counter if their entries are not maintained.

## 8. Recommended TTL Maintenance Practices

For future maintenance:

1. Define shared TTL constants rather than repeating numeric literals.
2. Review every persistent storage key and document its lifecycle.
3. Extend TTL after successful writes to persistent records.
4. Consider extending TTL after reads for records that should remain available while actively used.
5. Monitor long-running deliveries and disputed deliveries.
6. Ensure delivery-counter persistence is maintained.
7. Consider an archival or off-chain indexing strategy for historical records that no longer need to remain in contract storage.
8. Keep the authoritative state and any off-chain archival representation reconcilable.

Off-chain archival should not be treated as a replacement for required on-chain state. Applications should retain enough information to verify historical records against the blockchain.

## 9. Current Implementation Summary

| Area | Current behavior |
|---|---|
| Delivery record creation | Persistent write followed by TTL extension |
| Delivery record lifecycle updates | Persistent write followed by TTL extension |
| Delivery record reads | No explicit TTL extension |
| Driver profile update during confirmation | Persistent write followed by TTL extension |
| Driver profile reads | No explicit TTL extension |
| Delivery counter writes | No explicit TTL extension |
| Instance configuration | Stored in instance storage; not covered by the persistent-key extensions above |

## 10. Architecture Reference

See:

- `docs/architecture/smart-contract-architecture.md`
- `contracts/delivery_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `contracts/escrow_contract/lib.rs`

## Summary

The delivery contract already extends TTL for delivery records during creation and lifecycle updates, and for driver profiles when they are updated during confirmation. However, its read functions do not currently extend TTL, and the persistent delivery counter has no explicit TTL extension. These gaps matter for long-running deliveries and historical records. TTL maintenance and archival should therefore be considered part of the delivery contract's long-term storage strategy.
  
