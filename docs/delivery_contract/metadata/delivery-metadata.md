
# Delivery Metadata and Cargo Descriptor Types

## 1. Overview

SwiftChain uses shared data structures to represent delivery information and cargo characteristics.

These structures allow the delivery contract to store the information required to identify a delivery, describe its cargo, and track its expected delivery timeline.

The primary types are defined in:

`contracts/shared_types/lib.rs`

The delivery contract consumes these shared types through:

`contracts/delivery_contract/lib.rs`

This document explains the metadata structures, their fields, storage strategy, and the associated Soroban resource considerations.

## 2. Delivery Metadata Structure

The `DeliveryMetadata` structure is defined as:

```rust
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
```

### Field descriptions

| Field | Type | Description |
|---|---|---|
| `delivery_id` | `u64` | Unique delivery identifier represented by the metadata |
| `origin` | `String` | Origin location of the delivery |
| `destination` | `String` | Destination location of the delivery |
| `cargo_description` | `CargoDescriptor` | Structured description of the cargo |
| `created_at` | `u64` | Creation timestamp included in the metadata |
| `estimated_delivery` | `u64` | Estimated delivery timestamp |

### Important identifier distinction

The delivery contract generates a `DeliveryId` when a delivery is created.

The metadata also contains a `delivery_id` field.

The current `create_delivery` implementation stores the supplied metadata inside the delivery record but does not overwrite the metadata's `delivery_id` with the newly generated identifier.

Therefore, callers should supply a consistent identifier when constructing the metadata.

This distinction is important when integrating the delivery contract with off-chain applications.

## 3. Cargo Descriptor Structure

Cargo information is represented through the `CargoDescriptor` structure.

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoDescriptor {
    pub weight_grams: u32,
    pub category: CargoCategory,
    pub fragile: bool,
}
```

### Field descriptions

| Field | Type | Description |
|---|---|---|
| `weight_grams` | `u32` | Cargo weight expressed in grams |
| `category` | `CargoCategory` | Classification of the cargo |
| `fragile` | `bool` | Indicates whether the cargo requires fragile-item handling |

## 4. Cargo Categories

The `CargoCategory` enum defines the supported cargo classifications.

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CargoCategory {
    Documents,
    Electronics,
    Perishables,
    Clothing,
    General,
}
```

| Category | Description |
|---|---|
| `Documents` | Documents and paperwork |
| `Electronics` | Electronic devices and equipment |
| `Perishables` | Goods that may deteriorate over time |
| `Clothing` | Clothing and textile items |
| `General` | Other cargo that does not fit the specific categories |

The enum provides a structured classification rather than relying on arbitrary category strings.

## 5. Delivery Record Integration

The delivery metadata is embedded in the `DeliveryRecord` structure.

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryRecord {
    pub delivery_id: DeliveryId,
    pub sender: Address,
    pub recipient: Address,
    pub driver: Option<Address>,
    pub status: DeliveryStatus,
    pub metadata: DeliveryMetadata,
    pub created_at: u64,
    pub delivered_at: Option<u64>,
    pub transit_started_at: Option<u64>,
}
```

This structure combines delivery metadata with the information required to manage the delivery lifecycle.

The record includes:

- Delivery identifier.
- Sender address.
- Recipient address.
- Optional assigned driver.
- Delivery status.
- Cargo and location metadata.
- Delivery creation timestamp.
- Optional delivery completion timestamp.
- Optional transit start timestamp.

The `metadata` field is therefore part of the on-chain delivery record rather than an independently stored off-chain object.

## 6. On-Chain Storage Strategy

The delivery contract stores delivery records in Soroban persistent storage.

The relevant storage operation in `create_delivery` is:

```rust
let key = delivery_key(delivery_id);

env.storage().persistent().set(&key, &record);

env.storage()
    .persistent()
    .extend_ttl(&key, 518400, 518400);
```

The `delivery_key` helper is defined in:

`contracts/shared_types/lib.rs`

It produces a `StorageKey::Delivery` key for the delivery identifier.

### On-chain information

The current implementation stores the following information as part of the delivery record:

- Delivery identifier.
- Sender address.
- Recipient address.
- Assigned driver, when available.
- Delivery status.
- Origin.
- Destination.
- Cargo descriptor.
- Metadata creation timestamp.
- Estimated delivery timestamp.
- Delivery lifecycle timestamps.

The record is serialized into Soroban persistent storage.

### Storage lifetime

The delivery record uses persistent storage with a time-to-live extension.

The current contract uses:

```rust
extend_ttl(&key, 518400, 518400);
```

This represents the configured ledger-based TTL threshold and extension amount.

It should not be interpreted as a fixed number of seconds.

## 7. Off-Chain Data Strategy

The current `DeliveryMetadata` implementation does not contain:

- An off-chain metadata URI.
- An external object-storage identifier.
- An IPFS content identifier.
- A metadata content hash.
- A separate metadata document reference.

Consequently, the current contract does not implement an on-chain pointer to an independently stored metadata document.

An off-chain application may maintain additional delivery information in its database, but that information is not automatically part of the on-chain `DeliveryMetadata` structure.

### Recommended future extension

For larger delivery information requirements, an off-chain metadata strategy could be introduced.

A future design may store a compact reference or content hash on-chain while maintaining extended descriptive information in an external storage system.

Such a change would require an explicit contract schema and integration update. It is not part of the current implementation.

## 8. On-Chain vs Off-Chain Comparison

| Data | Current storage strategy |
|---|---|
| Delivery identifier | On-chain |
| Sender address | On-chain |
| Recipient address | On-chain |
| Assigned driver | On-chain |
| Delivery status | On-chain |
| Origin | On-chain |
| Destination | On-chain |
| Cargo weight | On-chain |
| Cargo category | On-chain |
| Fragile indicator | On-chain |
| Estimated delivery timestamp | On-chain |
| Additional application-specific notes | Not defined in the current contract metadata |
| External metadata URI | Not defined in the current contract metadata |
| External metadata content hash | Not defined in the current contract metadata |

An off-chain application may maintain additional information, but its existence and schema must be verified in the backend implementation.

## 9. Metadata Size and Soroban Resource Considerations

Storing metadata directly on-chain makes the delivery information available through contract storage and reduces dependence on an external metadata lookup for the fields already represented in the record.

However, persistent storage consumes Soroban resources.

The following considerations are important.

### 9.1 Storage footprint

The metadata contains:

- Several integer fields.
- Two Soroban strings.
- A nested cargo descriptor.
- An enum value.
- A boolean field.

The serialized storage footprint depends on the encoded values and their lengths.

The two location strings are particularly important because their storage requirements depend on their actual contents.

### 9.2 Transaction resource consumption

Creating or updating a delivery record requires serializing and persisting its contents.

Larger metadata values can increase:

- Storage write costs.
- Ledger storage footprint.
- Transaction resource consumption.
- Contract execution costs.

### 9.3 Persistent storage costs

Persistent storage has associated rent and TTL management considerations.

The contract extends the TTL of delivery records to support their continued availability.

The appropriate TTL strategy should account for the expected lifetime of delivery records and the protocol's storage requirements.

### 9.4 Off-chain storage trade-offs

Moving extended metadata off-chain can reduce the amount of information stored directly in Soroban persistent storage.

However, it introduces additional considerations:

- External data availability.
- Integrity verification.
- Metadata retrieval latency.
- Storage provider dependency.
- Content-addressing or hash verification.
- Off-chain synchronization.

A future off-chain strategy should retain sufficient on-chain information to identify and verify the relevant delivery data.

## 10. Metadata Validation Considerations

The current structures define the metadata types but do not, by themselves, establish all business validation rules.

Future enhancements may consider:

- Restricting empty origin and destination values.
- Defining maximum string lengths.
- Validating cargo weight.
- Checking estimated delivery timestamps.
- Ensuring the metadata delivery identifier matches the generated delivery identifier.
- Defining an explicit policy for metadata updates.

Such changes must be implemented and tested in the relevant contract functions before being documented as enforced behavior.

## 11. Source References

- `contracts/shared_types/lib.rs`
- `contracts/delivery_contract/lib.rs`
- `contracts/delivery_contract/test.rs`
- `docs/architecture/smart-contract-architecture.md`

## 12. Summary

SwiftChain represents delivery information using `DeliveryMetadata` and `CargoDescriptor`.

These types are embedded within `DeliveryRecord` and persisted in Soroban persistent storage by the delivery contract.

The current implementation stores the defined delivery metadata on-chain. It does not implement a separate off-chain metadata pointer or content-hash mechanism.

Understanding this distinction is important when extending the delivery schema, designing external application read models, and managing Soroban storage costs.
