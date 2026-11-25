
# SwiftChain Shared Core Data Types

## 1. Overview

SwiftChain uses a shared Rust library named `shared_types` to define protocol data structures and common types used by multiple smart-contract modules.

**Source:** `contracts/shared_types/lib.rs`

The shared crate provides common definitions for:

- Delivery identifiers and delivery records.
- Escrow records and escrow states.
- User and driver profiles.
- Delivery metadata and cargo descriptions.
- Protocol configuration.
- Party addresses.
- Shared storage keys.
- Event payload structures.
- Shared protocol errors.

The purpose of the shared crate is to reduce duplicated protocol definitions and avoid circular dependencies between contract modules.

The repository architecture describes seven core contracts and libraries. `shared_types` is the shared library in that architecture, not a seventh independently deployed contract.

## 2. Why shared_types exists

Without a shared crate, multiple contracts might independently define their own versions of delivery identifiers, escrow records, status enums, and other protocol structures.

That can lead to:

- Duplicated data definitions.
- Inconsistent field names and types.
- Incompatible serialized representations.
- Unnecessary dependencies between contract modules.
- Circular dependency problems.

The shared crate provides a common dependency direction:

```text
                    shared_types
                   /     |      \
                  /      |       \
                 v       v        v
          delivery   escrow   dispute
          contract   contract  resolution
                 \      |       /
                  \     |      /
                   other contracts
```

Each contract can import the common definitions it needs without importing another contract's implementation module merely to reuse a data structure.

This is a shared-code dependency pattern. It does not mean that all contracts share one physical Soroban storage instance.

## 3. Soroban serialization with contracttype

The shared data structures use Soroban's `#[contracttype]` attribute.

Example:

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

`#[contracttype]` makes the type compatible with Soroban's contract value and serialization system.

This allows supported structures and enums to be stored in contract storage or passed through contract interfaces.

The Rust derives such as `Clone`, `Debug`, `Eq`, and `PartialEq` provide ordinary Rust behavior. They do not replace Soroban's contract serialization support.

Changes to serialized types should be treated carefully because deployed contracts and clients may depend on the existing representation.

## 4. Core shared structures

### 4.1 DeliveryId

**Definition:** `contracts/shared_types/lib.rs`

```rust
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct DeliveryId(pub u64);
```

`DeliveryId` is a strongly typed wrapper around a `u64`.

| Field | Type | Meaning |
|---|---|---|
| `0` | `u64` | The underlying numeric delivery identifier. |

The wrapper provides methods for creating and reading identifiers:

```rust
impl DeliveryId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}
```

It also implements conversions between `DeliveryId` and `u64`.

This allows contract code to distinguish delivery identifiers from arbitrary numeric values while retaining compatibility with the underlying integer.

### 4.2 DeliveryRecord

**Definition:** `contracts/shared_types/lib.rs`

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

| Field | Type | Meaning |
|---|---|---|
| `delivery_id` | `DeliveryId` | Unique identifier for the delivery. |
| `sender` | `Address` | Address of the user who created the delivery. |
| `recipient` | `Address` | Address of the intended recipient. |
| `driver` | `Option<Address>` | Assigned driver's address, or `None` when no driver is assigned. |
| `status` | `DeliveryStatus` | Current delivery lifecycle state. |
| `metadata` | `DeliveryMetadata` | Delivery origin, destination, cargo description, and timing metadata. |
| `created_at` | `u64` | Ledger timestamp associated with delivery creation. |
| `delivered_at` | `Option<u64>` | Delivery completion timestamp, if available. |
| `transit_started_at` | `Option<u64>` | Timestamp when transit began, if available. |

**Primary consumer:** `delivery_contract`.

The delivery contract creates and updates delivery records and stores them using the shared delivery storage key.

The dispute-resolution contract also reads delivery records through the delivery contract interface.

### 4.3 EscrowRecord

**Definition:** `contracts/shared_types/lib.rs`

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowRecord {
    pub sender: Address,
    pub recipient: Address,
    pub driver: Address,
    pub token: Address,
    pub amount: i128,
    pub status: EscrowState,
    pub created_at: u64,
    pub disputed_by: Option<Address>,
    pub disputed_at: Option<u64>,
}
```

| Field | Type | Meaning |
|---|---|---|
| `sender` | `Address` | Address that funded the escrow. |
| `recipient` | `Address` | Intended recipient associated with the delivery. |
| `driver` | `Address` | Driver address associated with the escrow payout. |
| `token` | `Address` | Token contract address used for the escrowed asset. |
| `amount` | `i128` | Escrow amount in the token's smallest unit. |
| `status` | `EscrowState` | Current escrow lifecycle state. |
| `created_at` | `u64` | Ledger timestamp when the escrow record was created. |
| `disputed_by` | `Option<Address>` | Address that raised the dispute, when present. |
| `disputed_at` | `Option<u64>` | Timestamp associated with the dispute, when present. |

**Primary consumer:** `escrow_contract`.

The escrow contract stores records using the shared `StorageKey::Escrow` key and uses the record during funding, release, refund, and dispute-resolution operations.

The dispute-resolution contract also reads escrow records through the escrow contract interface.

### 4.4 UserProfile

**Definition:** `contracts/shared_types/lib.rs`

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub address: Address,
    pub registered_at: u64,
}
```

| Field | Type | Meaning |
|---|---|---|
| `address` | `Address` | User's Soroban address. |
| `registered_at` | `u64` | Registration timestamp represented as a ledger timestamp. |

This is the shared user profile definition.

**Important implementation distinction:** `contracts/identity_reputation_contract/lib.rs` currently defines its own local `UserProfile` with a `join_date` field instead of importing this shared definition.

The local identity-contract structure and this shared structure should not be described as identical Rust types.

### 4.5 DriverProfile

**Definition:** `contracts/shared_types/lib.rs`

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriverProfile {
    pub address: Address,
    pub deliveries_completed: u32,
    pub reputation_score: u32,
    pub registered_at: u64,
    pub kyc_verified: bool,
}
```

| Field | Type | Meaning |
|---|---|---|
| `address` | `Address` | Driver's Soroban address. |
| `deliveries_completed` | `u32` | Number of completed deliveries recorded for the driver. |
| `reputation_score` | `u32` | Driver reputation score. |
| `registered_at` | `u64` | Driver registration timestamp. |
| `kyc_verified` | `bool` | Indicates whether the driver has passed the recorded KYC verification state. |

The delivery contract imports the shared `DriverProfile` type when updating driver information.

**Important implementation distinction:** `contracts/identity_reputation_contract/lib.rs` also defines a local `DriverProfile` with the same field names and types. It is a separate module-local Rust definition.

The shared and local types should not be assumed to be interchangeable without an explicit conversion or a future source refactor.

### 4.6 DeliveryMetadata

**Definition:** `contracts/shared_types/lib.rs`

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

| Field | Type | Meaning |
|---|---|---|
| `delivery_id` | `u64` | Numeric identifier associated with the delivery metadata. |
| `origin` | `String` | Delivery pickup or origin description. |
| `destination` | `String` | Delivery destination description. |
| `cargo_description` | `CargoDescriptor` | Structured description of the cargo. |
| `created_at` | `u64` | Metadata creation timestamp. |
| `estimated_delivery` | `u64` | Estimated delivery timestamp. |

**Consumer:** `delivery_contract`, through `DeliveryRecord`.

### 4.7 CargoDescriptor

**Definition:** `contracts/shared_types/lib.rs`

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoDescriptor {
    pub weight_grams: u32,
    pub category: CargoCategory,
    pub fragile: bool,
}
```

| Field | Type | Meaning |
|---|---|---|
| `weight_grams` | `u32` | Cargo weight in grams. |
| `category` | `CargoCategory` | Classification of the cargo. |
| `fragile` | `bool` | Indicates whether the cargo is marked as fragile. |

### 4.8 PartyAddresses

**Definition:** `contracts/shared_types/lib.rs`

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartyAddresses {
    pub sender: Address,
    pub driver: Address,
    pub recipient: Address,
}
```

| Field | Type | Meaning |
|---|---|---|
| `sender` | `Address` | Delivery sender address. |
| `driver` | `Address` | Delivery driver address. |
| `recipient` | `Address` | Delivery recipient address. |

This structure groups the three primary participant addresses associated with a delivery.

### 4.9 ProtocolConfig

**Definition:** `contracts/shared_types/lib.rs`

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolConfig {
    pub token: Address,
    pub platform_fee_bps: u32,
    pub protocol_version: u32,
}
```

| Field | Type | Meaning |
|---|---|---|
| `token` | `Address` | Configured token contract address. |
| `platform_fee_bps` | `u32` | Platform fee expressed in basis points. |
| `protocol_version` | `u32` | Protocol configuration version. |

The escrow contract reads and writes this configuration through shared `StorageKey::ProtocolConfig`.

### 4.10 DeliveryDetails

**Definition:** `contracts/shared_types/lib.rs`

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryDetails {
    pub id: DeliveryId,
    pub driver: String,
    pub status: DeliveryStatus,
}
```

| Field | Type | Meaning |
|---|---|---|
| `id` | `DeliveryId` | Delivery identifier. |
| `driver` | `String` | Driver value represented as a Soroban string. |
| `status` | `DeliveryStatus` | Delivery lifecycle state. |

This is an additional shared contract type. It should not be confused with the more detailed `DeliveryRecord`.

## 5. Shared event payload structures

The shared crate also defines contract-serializable event payload structures.

These are data structures for event information, not the primary delivery or escrow storage records.

| Structure | Fields | Purpose |
|---|---|---|
| `DeliveryCreatedEvent` | `delivery_id: u64`, `sender: Address`, `amount: i128` | Describes a delivery creation event. |
| `EscrowFundedEvent` | `delivery_id: u64`, `sender: Address`, `token: Address`, `amount: i128` | Describes an escrow funding event. |
| `DriverAssignedEvent` | `delivery_id: u64`, `driver: Address` | Describes driver assignment. |
| `DeliveryConfirmedEvent` | `delivery_id: u64`, `recipient: Address`, `timestamp: u64` | Describes delivery confirmation. |
| `EscrowReleasedEvent` | `delivery_id: u64`, `driver: Address`, `amount: i128`, `platform_fee: i128` | Describes escrow release. |
| `DeliveryDisputedEvent` | `delivery_id: u64`, `reporter: Address`, `timestamp: u64` | Describes a delivery dispute. |
| `EscrowRefundedEvent` | `delivery_id: u64`, `sender: Address`, `amount: i128` | Describes an escrow refund. |

Each event structure is annotated with `#[contracttype]`.

The actual event publication sites and their topic/payload combinations should be checked in the contract that emits the event. Defining a shared event structure does not mean every contract uses that structure as its event payload.

## 6. Shared enums

### DeliveryStatus

```rust
pub enum DeliveryStatus {
    Pending,
    Active,
    InTransit,
    Delivered,
    Disputed,
    Cancelled,
}
```

Represents the lifecycle state of a delivery.

### EscrowState

```rust
pub enum EscrowState {
    Locked,
    Released,
    Refunded,
    Paused,
}
```

Represents the lifecycle state of an escrow record.

`EscrowStatus` is a type alias for `EscrowState`.

### CargoCategory

```rust
pub enum CargoCategory {
    Documents,
    Electronics,
    Perishables,
    Clothing,
    General,
}
```

Represents the supported cargo classifications.

## 7. Types that are not currently defined in shared_types

The issue description names `FleetProfile` and `DisputeCase` among the structures to document.

However, the current source places these structures in their respective contract modules.

### FleetProfile

**Actual source:** `contracts/fleet_management_contract/lib.rs`

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FleetProfile {
    pub fleet_id: FleetId,
    pub owner: Address,
    pub treasury: Address,
    pub total_active_drivers: u32,
}
```

| Field | Type | Meaning |
|---|---|---|
| `fleet_id` | `FleetId` (`u64`) | Unique fleet identifier. |
| `owner` | `Address` | Fleet owner's address. |
| `treasury` | `Address` | Wallet address configured to receive fleet payouts when routing is used. |
| `total_active_drivers` | `u32` | Number of active drivers associated with the fleet. |

This is a fleet-management contract-local type, not a type in `shared_types`.

### DisputeCase

**Actual source:** `contracts/dispute_resolution_contract/lib.rs`

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeCase {
    pub delivery_id: DeliveryId,
    pub status: DisputeStatus,
    pub raised_at: u64,
    pub raised_by: Address,
    pub evidence_hashes: Vec<BytesN<32>>,
}
```

| Field | Type | Meaning |
|---|---|---|
| `delivery_id` | `DeliveryId` | Delivery associated with the dispute. |
| `status` | `DisputeStatus` | Current dispute state. |
| `raised_at` | `u64` | Timestamp when the dispute was raised. |
| `raised_by` | `Address` | Address that raised the dispute. |
| `evidence_hashes` | `Vec<BytesN<32>>` | Collection of 32-byte evidence hashes. |

This is a dispute-resolution contract-local type, not a type in `shared_types`.

### Other contract-local profile definitions

The identity/reputation contract defines its own local `UserProfile` and `DriverProfile` structures.

These are separate from the shared profile structures described earlier.

This distinction is important when changing fields, serializing records, or passing values between contracts.

## 8. Consuming contract reference

| Type | Source definition | Main consumer or usage |
|---|---|---|
| `DeliveryId` | `shared_types` | Delivery, escrow, and dispute-resolution workflows. |
| `DeliveryRecord` | `shared_types` | Delivery contract; read by dispute resolution through the delivery interface. |
| `EscrowRecord` | `shared_types` | Escrow contract; read by dispute resolution through the escrow interface. |
| `UserProfile` | `shared_types` | Shared definition; identity contract currently has a local definition. |
| `DriverProfile` | `shared_types` | Delivery contract imports the shared definition; identity contract has a local definition. |
| `DeliveryMetadata` | `shared_types` | Embedded in `DeliveryRecord` and used by the delivery contract. |
| `CargoDescriptor` | `shared_types` | Embedded in `DeliveryMetadata`. |
| `CargoCategory` | `shared_types` | Used by `CargoDescriptor`. |
| `PartyAddresses` | `shared_types` | Shared participant-address grouping. |
| `ProtocolConfig` | `shared_types` | Escrow contract configuration. |
| `DeliveryDetails` | `shared_types` | Additional shared delivery representation. |
| `FleetProfile` | Fleet management contract | Fleet registration, treasury updates, and fleet queries. |
| `DisputeCase` | Dispute resolution contract | Dispute creation, evidence, and resolution. |

## 9. Compatibility and contributor guidelines

When modifying shared structures:

1. Check all contracts importing the type.
2. Check storage reads and writes that serialize the type.
3. Check cross-contract calls that pass the type.
4. Consider compatibility with already-deployed contract state.
5. Avoid changing field names or types without reviewing serialization compatibility.
6. Keep shared definitions in `shared_types` when multiple contracts genuinely need the same type.
7. Keep contract-specific structures local when they are only relevant to one contract.
8. Update this document when a shared type, field, enum variant, or consumer changes.

Do not move `FleetProfile`, `DisputeCase`, or the identity contract's local profile types into `shared_types` merely to make the documentation appear consistent. Such a refactor would be a separate code change requiring dependency and serialization review.

## 10. Source references

- `contracts/shared_types/lib.rs`
- `contracts/delivery_contract/lib.rs`
- `contracts/escrow_contract/lib.rs`
- `contracts/fleet_management_contract/lib.rs`
- `contracts/dispute_resolution_contract/lib.rs`
- `contracts/identity_reputation_contract/lib.rs`
- `docs/architecture/smart-contract-architecture.md`
