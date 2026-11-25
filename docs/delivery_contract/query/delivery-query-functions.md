
# Delivery Query Functions and Off-Chain Read Models

## Overview

The `delivery_contract` exposes read-only query functions that allow clients and other application components to retrieve delivery information and driver profile information.

The query functions covered in this document are:

- `get_delivery`
- `get_driver_profile`

These functions read data from the delivery contract's persistent storage. They do not create or update delivery records.

The implementation is located in:

- `contracts/delivery_contract/lib.rs`
- `contracts/shared_types/lib.rs`

## 1. get_delivery

### Function Signature

```rust
pub fn get_delivery(env: Env, delivery_id: DeliveryId) -> DeliveryRecord
```

### Purpose

`get_delivery` retrieves a previously stored delivery record using its unique delivery identifier.

The function constructs the delivery storage key with `delivery_key(delivery_id)` and reads the corresponding record from persistent contract storage.

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment used to access contract storage. |
| `delivery_id` | `DeliveryId` | Unique identifier of the delivery to retrieve. |

### Return Type

The function returns:

```rust
DeliveryRecord
```

`DeliveryRecord` is defined in `contracts/shared_types/lib.rs`.

### DeliveryRecord Fields

| Field | Type | Description |
|---|---|---|
| `delivery_id` | `DeliveryId` | Unique identifier assigned to the delivery. |
| `sender` | `Address` | Address that created the delivery. |
| `recipient` | `Address` | Address expected to receive the delivery. |
| `driver` | `Option<Address>` | Assigned driver's address, or `None` when no driver has been assigned. |
| `status` | `DeliveryStatus` | Current delivery lifecycle status. |
| `metadata` | `DeliveryMetadata` | Delivery information supplied when the delivery was created. |
| `created_at` | `u64` | Ledger timestamp when the delivery record was created. |
| `delivered_at` | `Option<u64>` | Ledger timestamp when delivery was confirmed, or `None` if it has not been confirmed. |
| `transit_started_at` | `Option<u64>` | Ledger timestamp when the delivery entered transit, or `None` if transit has not started. |

### DeliveryStatus Values

The shared `DeliveryStatus` enum defines these states:

- `Pending`
- `Active`
- `InTransit`
- `Delivered`
- `Disputed`
- `Cancelled`

The current status is stored in the delivery record and returned by `get_delivery`.

### DeliveryMetadata Fields

The `metadata` field contains a `DeliveryMetadata` structure:

| Field | Type | Description |
|---|---|---|
| `delivery_id` | `u64` | Delivery identifier included in the supplied metadata. |
| `origin` | `String` | Delivery origin. |
| `destination` | `String` | Delivery destination. |
| `cargo_description` | `CargoDescriptor` | Cargo weight, category, and fragility information. |
| `created_at` | `u64` | Timestamp supplied in the metadata. |
| `estimated_delivery` | `u64` | Estimated delivery timestamp supplied in the metadata. |

`CargoDescriptor` contains:

| Field | Type | Description |
|---|---|---|
| `weight_grams` | `u32` | Cargo weight in grams. |
| `category` | `CargoCategory` | Cargo classification. |
| `fragile` | `bool` | Indicates whether the cargo is fragile. |

The available `CargoCategory` variants are:

- `Documents`
- `Electronics`
- `Perishables`
- `Clothing`
- `General`

### Storage and Missing Records

The function uses:

```rust
let key = delivery_key(delivery_id);
```

It then retrieves the record from persistent storage:

```rust
env.storage()
    .persistent()
    .get(&key)
    .unwrap_or_else(|| panic!("DeliveryNotFound"))
```

If the requested delivery does not exist, the function panics with `DeliveryNotFound`.

The query does not return an optional value and does not create a missing delivery record.

## 2. get_driver_profile

### Function Signature

```rust
pub fn get_driver_profile(env: Env, driver: Address) -> DriverProfile
```

### Purpose

`get_driver_profile` retrieves the driver profile stored in the delivery contract's persistent storage.

It uses the shared `StorageKey::DriverProfile` variant to construct the storage key.

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment used to access contract storage. |
| `driver` | `Address` | Address of the driver whose profile is requested. |

### Return Type

The function returns:

```rust
DriverProfile
```

The returned structure is defined in `contracts/shared_types/lib.rs`.

### DriverProfile Fields

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | Driver's account address. |
| `deliveries_completed` | `u32` | Number of completed deliveries recorded in this profile. |
| `reputation_score` | `u32` | Reputation score recorded in this profile. |
| `registered_at` | `u64` | Ledger timestamp used when the fallback profile is constructed. |
| `kyc_verified` | `bool` | KYC verification status recorded in the profile. |

### Storage Lookup and Fallback Behavior

The function constructs its key:

```rust
let driver_key = StorageKey::DriverProfile(driver.clone());
```

It then attempts to retrieve the profile from persistent storage.

If no profile exists under that key, it returns a default in-memory profile with:

| Field | Fallback value |
|---|---|
| `address` | Requested driver address |
| `deliveries_completed` | `0` |
| `reputation_score` | `0` |
| `registered_at` | Current ledger timestamp |
| `kyc_verified` | `false` |

Important: this fallback profile is returned by the query. The function does not persist the fallback profile merely because it was requested.

### Relationship with the Identity and Reputation Contract

The delivery contract and identity/reputation contract both use a `DriverProfile` data structure, but their storage is contract-specific.

The identity/reputation contract maintains driver profiles under its own persistent storage and initializes registered drivers with a reputation score of `50`.

The delivery contract's `get_driver_profile` function reads from the delivery contract's own storage. Its fallback profile starts with a reputation score of `0`.

These values should not be treated as interchangeable. An application that needs the authoritative identity and reputation profile should query the identity/reputation contract directly.

The delivery contract also updates its locally stored driver profile during delivery confirmation when an assigned driver exists.

## 3. Off-Chain Read-Model Implications

A read model is an application-side representation of on-chain data that is optimized for querying, filtering, reporting, and displaying information.

The contract query functions provide the source data, while an off-chain read model can organize that data for application use.

### Why an Off-Chain Read Model Is Useful

A production logistics application may need to:

- Display delivery details to senders and recipients.
- Show a driver's current assigned deliveries.
- Filter deliveries by status.
- Display delivery history.
- Generate operational reports.
- Support dashboards and administrative search.
- Track delivery lifecycle timestamps.
- Build notifications when relevant contract events are observed.

Repeatedly retrieving and processing many individual contract records may not be the most efficient approach for these application requirements.

An off-chain read model can index relevant contract data and expose application-friendly query endpoints.

### Recommended Data Flow

1. A user or authorized application component submits a transaction to the appropriate contract function.
2. Soroban executes the transaction and updates contract storage.
3. The application or indexing service observes relevant on-chain events and/or retrieves the updated contract state.
4. The indexing service updates its application-side read model.
5. The frontend retrieves the indexed information through an application API.
6. The application can periodically reconcile indexed records against contract state.

### Suggested Read-Model Fields

A delivery read model may include:

| Field | Purpose |
|---|---|
| `deliveryId` | Lookup and identification. |
| `sender` | Sender filtering and display. |
| `recipient` | Recipient filtering and display. |
| `driver` | Driver assignment display. |
| `status` | Status filtering and dashboards. |
| `origin` | Origin display and filtering. |
| `destination` | Destination display and filtering. |
| `createdAt` | Delivery creation history. |
| `transitStartedAt` | Transit tracking. |
| `deliveredAt` | Completion tracking. |
| `lastIndexedLedger` | Indexing progress and reconciliation. |

These are suggested application-side fields, not additional fields in the on-chain `DeliveryRecord`.

### Events and Reconciliation

Events can help an off-chain indexer identify relevant state changes. The delivery contract currently publishes events for operations such as:

- `delivery_created`
- `driver_assigned`
- `DeliveryInTransit`
- `delivery_confirmed`
- `delivery_cancelled`
- `delivery_disputed`

The exact event topics and payloads should be read from the contract implementation.

Events are useful for triggering read-model updates, but an indexer should not assume that an event stream alone is a complete substitute for contract state.

A robust indexing service should support:

- Reprocessing events safely.
- Handling duplicate event delivery.
- Tracking the last processed ledger.
- Recovering from missed indexing intervals.
- Reconciling indexed records with contract storage.
- Handling records that are no longer available because of storage lifecycle or retention constraints.

### Source of Truth

The Soroban contract state is the authoritative source for on-chain delivery records.

An off-chain read model is a derived representation. It should be treated as a query optimization and not as an independent authority for deciding whether a delivery was completed, cancelled, or disputed.

For sensitive operations, the application should verify the relevant on-chain state rather than relying exclusively on cached or indexed data.

## 4. Important Implementation Notes

- `get_delivery` returns a `DeliveryRecord` directly and panics when the delivery key is missing.
- `get_driver_profile` returns a `DriverProfile` directly and constructs a fallback profile when its local key is missing.
- Both functions read persistent storage.
- Neither query function requires an address authorization argument.
- The delivery contract's driver profile storage is separate from the identity/reputation contract's profile storage.
- Off-chain read models should be reconciled against on-chain state.

## 5. Source References

- `contracts/delivery_contract/lib.rs`
- `contracts/delivery_contract/test.rs`
- `contracts/shared_types/lib.rs`
- `contracts/identity_reputation_contract/lib.rs`
- `docs/architecture/smart-contract-architecture.md`
