
# SwiftChain Storage Key Namespaces

## 1. Overview

SwiftChain uses Soroban contract storage to persist protocol configuration, delivery records, escrow records, user and driver profiles, fleet information, dispute records, and settlement configuration.

Storage keys are represented by Rust enums annotated with `#[contracttype]`. These enums provide structured keys that Soroban can serialize and use when accessing contract storage.

The repository uses two related key patterns:

- `shared_types::StorageKey`: shared key definitions used by multiple contracts.
- Contract-local `DataKey` enums: keys defined inside individual contract modules for contract-specific state.

Each deployed Soroban contract has its own storage context. Consequently, an identical enum variant or logical key in two different contract instances does not automatically refer to the same stored entry.

## 2. Shared StorageKey

**Source:** `contracts/shared_types/lib.rs`

The shared library defines:

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    Admin,
    Delivery(DeliveryId),
    Escrow(DeliveryId),
    DriverProfile(Address),
    ProtocolConfig,
}
```

The variants are:

| Variant | Key data | Intended storage category | Purpose |
|---|---|---|---|
| `Admin` | None | Instance | Stores the shared contract administrator address. |
| `Delivery(DeliveryId)` | Delivery identifier | Persistent | Identifies a delivery record. |
| `Escrow(DeliveryId)` | Delivery identifier | Persistent | Identifies an escrow record. |
| `DriverProfile(Address)` | Driver address | Persistent | Identifies a driver profile. |
| `ProtocolConfig` | None | Instance | Stores protocol configuration such as token address, platform fee basis points, and protocol version. |

The helper functions `delivery_key` and `escrow_key` construct the corresponding shared keys:

```rust
pub fn delivery_key(id: impl Into<DeliveryId>) -> StorageKey {
    StorageKey::Delivery(id.into())
}

pub fn escrow_key(id: impl Into<DeliveryId>) -> StorageKey {
    StorageKey::Escrow(id.into())
}
```

These helpers reduce repeated key-construction logic and help consumers use consistent key representations.

## 3. Contract-local DataKey enums

The following sections document the `DataKey` enums currently present in the contract source.

### 3.1 Delivery contract

**Source:** `contracts/delivery_contract/lib.rs`

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    DeliveryCounter,
    EscrowContract,
}
```

| Variant | Storage category | Purpose |
|---|---|---|
| `DeliveryCounter` | Instance | Tracks the delivery identifier counter used when creating deliveries. |
| `EscrowContract` | Instance | Stores the configured escrow contract address used for cross-contract calls. |

The delivery counter is initialized and accessed through instance storage.

Delivery records themselves use the shared `StorageKey::Delivery(DeliveryId)` key and persistent storage.

The delivery contract extends the TTL of delivery records after writing or updating them.

### 3.2 Escrow contract

**Source:** `contracts/escrow_contract/lib.rs`

```rust
#[contracttype]
#[derive(Clone)]
enum DataKey {
    PendingAdmin,
    SettlementContract,
}
```

| Variant | Storage category | Purpose |
|---|---|---|
| `PendingAdmin` | Instance | Stores the proposed administrator during the administrator transfer process. |
| `SettlementContract` | Instance | Stores the optional settlement contract address used during driver payout. |

The escrow contract uses shared storage keys for its main records:

- `StorageKey::Admin`
- `StorageKey::ProtocolConfig`
- `StorageKey::Escrow(DeliveryId)`

Escrow records are stored in persistent storage through `escrow_key(delivery_id)`.

The local `DataKey` enum is used for additional escrow-specific configuration.

### 3.3 Dispute resolution contract

**Source:** `contracts/dispute_resolution_contract/lib.rs`

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin(Address),
    DeliveryContract,
    EscrowContract,
    IdentityReputationContract,
    DisputeTimeLimit,
    Dispute(DeliveryId),
}
```

| Variant | Storage category | Purpose |
|---|---|---|
| `Admin(Address)` | Instance | Represents an administrator entry keyed by the administrator address. |
| `DeliveryContract` | Instance | Stores the configured delivery contract address. |
| `EscrowContract` | Instance | Stores the configured escrow contract address. |
| `IdentityReputationContract` | Instance | Stores the configured identity and reputation contract address. |
| `DisputeTimeLimit` | Instance | Stores the configured dispute time limit. |
| `Dispute(DeliveryId)` | Persistent | Identifies a dispute record associated with a delivery. |

Dispute records are written to persistent storage and have their TTL extended after relevant updates.

The contract configuration and administrator entries use instance storage.

### 3.4 Fleet management contract

**Source:** `contracts/fleet_management_contract/lib.rs`

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    IdentityContract,
    FleetCounter,
    Fleet(FleetId),
    DriverFleet(FleetId, Address),
}
```

| Variant | Storage category | Purpose |
|---|---|---|
| `Admin` | Instance | Stores the fleet-management administrator address. |
| `IdentityContract` | Instance | Stores the optional identity/reputation contract address. |
| `FleetCounter` | Persistent | Tracks the fleet identifier counter. |
| `Fleet(FleetId)` | Persistent | Stores a fleet profile keyed by fleet identifier. |
| `DriverFleet(FleetId, Address)` | Persistent | Stores the relationship/status of a driver within a fleet. |

Fleet profiles and driver-to-fleet relationships use persistent storage.

The fleet-management contract extends TTL for fleet and driver-fleet entries when they are written or updated.

### 3.5 Identity and reputation contract

**Source:** `contracts/identity_reputation_contract/lib.rs`

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    UserProfile(Address),
    DriverProfile(Address),
    AuthorizedContract(Address),
    DeliveryContract,
    DisputeContract,
}
```

| Variant | Storage category | Purpose |
|---|---|---|
| `Admin` | Instance | Stores the identity contract administrator address. |
| `UserProfile(Address)` | Persistent | Identifies a user profile by user address. |
| `DriverProfile(Address)` | Persistent | Identifies a driver profile by driver address. |
| `AuthorizedContract(Address)` | Persistent | Stores whether a contract address is authorized to invoke protected operations. |
| `DeliveryContract` | Instance | Stores the configured delivery contract address. |
| `DisputeContract` | Instance | Stores the configured dispute-resolution contract address. |

User and driver profile records are stored persistently.

The contract extends profile TTL when profiles are registered or updated.

### 3.6 Settlement contract

**Source:** `contracts/settlement_contract/lib.rs`

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
enum DataKey {
    Admin,
    Router,
    EscrowContract,
    Xlm,
    Usdc,
    Ngnc,
    Kes,
    AssetPair(Address, Address),
    DriverPreference(Address),
    PlatformFeeBps,
}
```

| Variant | Storage category | Purpose |
|---|---|---|
| `Admin` | Instance | Stores the settlement administrator address. |
| `Router` | Instance | Stores the configured swap router address. |
| `EscrowContract` | Instance | Stores the authorized escrow contract address. |
| `Xlm` | Instance | Stores the supported-asset flag for XLM. |
| `Usdc` | Instance | Stores the supported-asset flag for USDC. |
| `Ngnc` | Instance | Stores the supported-asset flag for NGNC. |
| `Kes` | Instance | Stores the supported-asset flag for KES. |
| `AssetPair(Address, Address)` | Persistent | Identifies a supported source and destination asset pair. |
| `DriverPreference(Address)` | Instance | Identifies the preferred settlement asset for a driver. |
| `PlatformFeeBps` | Instance | Stores the settlement platform fee in basis points. |

The supported asset flags, router configuration, escrow address, driver preferences, and platform fee are accessed through instance storage in the current implementation.

Supported asset-pair entries use persistent storage.

## 4. Persistent, instance, and temporary storage

Soroban exposes different storage classes with different lifetimes and intended use cases.

### Persistent storage

Persistent storage is suitable for records that should remain available across contract invocations and can be retained by extending their TTL.

The repository uses persistent storage for examples such as:

- Delivery records.
- Escrow records.
- Dispute records.
- User and driver profiles.
- Fleet profiles.
- Driver-to-fleet relationships.
- Authorized contract entries.
- Supported settlement asset pairs.
- Fleet counters.

Persistent entries can expire when their TTL is not maintained.

### Instance storage

Instance storage is associated with the contract instance and is appropriate for configuration and state that belongs to the deployed contract.

The repository uses instance storage for examples such as:

- Administrator addresses.
- Protocol configuration.
- Contract addresses used for cross-contract calls.
- Settlement router configuration.
- Platform fee configuration.
- Delivery counter.
- Dispute time-limit configuration.

Instance storage has its own TTL behavior. The escrow contract explicitly extends instance storage TTL during administrator-transfer operations.

### Temporary storage

Temporary storage is available in Soroban for short-lived data that does not need persistent retention.

The inspected contract modules do not currently call `env.storage().temporary()`.

Therefore, temporary storage should not be documented as an existing storage mechanism in the current SwiftChain implementation.

If temporary storage is introduced later, its key definitions, expiry expectations, and read/write behavior should be documented separately.

## 5. TTL extension

Soroban storage entries have a time-to-live (TTL) measured in ledger sequence numbers, not seconds.

The repository commonly uses:

```rust
env.storage()
    .persistent()
    .extend_ttl(&key, 518400, 518400);
```

The escrow contract defines:

```rust
pub mod constants {
    pub const ESCROW_TTL_THRESHOLD: u32 = 518400;
    pub const ESCROW_TTL_EXTEND_TO: u32 = 518400;
    pub const PROTOCOL_VERSION: u32 = 1;
}
```

The two TTL parameters have distinct meanings:

- `threshold`: the remaining TTL level at which an extension should be applied.
- `extend_to`: the target TTL level after extension.

The value `518400` represents a number of ledgers. It must not be described as 518,400 seconds or as a guaranteed number of days.

### Persistent-entry TTL behavior

The contracts extend persistent-entry TTL after operations such as:

- Creating or updating delivery records.
- Creating or updating escrow records.
- Raising or updating disputes.
- Registering or updating driver profiles.
- Registering or updating user profiles.
- Creating or updating fleet profiles.
- Updating driver-fleet relationships.

The escrow contract also extends the TTL when loading an escrow record.

### Instance-entry TTL behavior

The escrow contract extends instance storage TTL during administrator proposal and acceptance operations.

Instance storage TTL extension is performed through the instance storage API rather than by passing an individual persistent key.

## 6. Namespace isolation and collision prevention

Storage keys are interpreted within the storage of the contract instance that accesses them.

For example:

- `StorageKey::Delivery(DeliveryId(10))` in the delivery contract is not automatically the same storage entry as a key with the same logical value in the escrow contract.
- `DataKey::Admin` in the settlement contract is isolated from `DataKey::Admin` in the fleet-management contract.
- A shared Rust enum provides consistent key definitions, but it does not merge the storage of separate deployed contracts.

The namespace scheme therefore relies on:

1. Contract-instance storage isolation.
2. Typed key enums.
3. Distinct enum variants for different record categories.
4. Identifier or address values for record-specific keys.
5. Shared key helpers where multiple contracts need consistent key construction.

## 7. Contributor guidance

When adding a new storage entry:

1. Decide whether the value belongs in instance, persistent, or temporary storage.
2. Add the key to the appropriate contract-local `DataKey` or shared `StorageKey`.
3. Use a typed identifier or address where the record is associated with a specific entity.
4. Keep storage reads and writes consistent with the selected storage category.
5. Extend TTL for persistent entries where the record must remain available.
6. Document the key, storage category, value, and lifecycle in this reference.
7. Avoid changing existing key variants or their serialized representation without considering deployed-contract compatibility.

## 8. Source references

- `contracts/shared_types/lib.rs`
- `contracts/delivery_contract/lib.rs`
- `contracts/escrow_contract/lib.rs`
- `contracts/dispute_resolution_contract/lib.rs`
- `contracts/fleet_management_contract/lib.rs`
- `contracts/identity_reputation_contract/lib.rs`
- `contracts/settlement_contract/lib.rs`
- `docs/architecture/smart-contract-architecture.md`
