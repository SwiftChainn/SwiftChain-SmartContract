
# Identity and Reputation Contract Profile Structures

## Overview

The Identity and Reputation Contract defines the data structures used to represent registered users and drivers.

The two primary profile structures are:

- `UserProfile`
- `DriverProfile`

These structures are defined in:

```text
contracts/identity_reputation_contract/lib.rs
```

The contract also defines storage keys used to identify and retrieve these profiles.

This document describes the fields, storage model, and default values based on the current contract implementation.

## 1. UserProfile

### Struct Definition

```rust
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct UserProfile {
    pub address: Address,
    pub join_date: u64,
}
```

### Purpose

`UserProfile` stores the basic identity information of a registered SwiftChain user.

It records the user's blockchain address and the ledger timestamp associated with registration.

### Fields

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | Blockchain address associated with the registered user. |
| `join_date` | `u64` | Soroban ledger timestamp recorded when the user registers. |

### Default Values on Creation

The `register_user` function creates the profile using:

```rust
let join_date = env.ledger().timestamp();

let profile = UserProfile {
    address: user.clone(),
    join_date,
};
```

Therefore:

- `address` is set to the address supplied to `register_user`.
- `join_date` is set to the current ledger timestamp.

The contract does not define a separate default reputation score or driver tier for a `UserProfile`.

## 2. DriverProfile

### Struct Definition

```rust
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DriverProfile {
    pub address: Address,
    pub deliveries_completed: u32,
    pub reputation_score: u32,
    pub registered_at: u64,
    pub kyc_verified: bool,
}
```

### Purpose

`DriverProfile` stores the identity, delivery activity, reputation, and KYC status associated with a registered driver.

### Fields

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | Blockchain address associated with the driver. |
| `deliveries_completed` | `u32` | Number of completed deliveries recorded for the driver. |
| `reputation_score` | `u32` | Driver reputation score. |
| `registered_at` | `u64` | Soroban ledger timestamp when the driver profile was registered. |
| `kyc_verified` | `bool` | Indicates whether the driver's KYC status has been marked as verified. |

### Default Values on Creation

The `register_driver` function initializes a driver profile with:

```rust
let profile = DriverProfile {
    address: driver.clone(),
    deliveries_completed: 0,
    reputation_score: 50,
    registered_at: env.ledger().timestamp(),
    kyc_verified: false,
};
```

The defaults are:

| Field | Initial value |
|---|---|
| `address` | Supplied driver address |
| `deliveries_completed` | `0` |
| `reputation_score` | `50` |
| `registered_at` | Current ledger timestamp |
| `kyc_verified` | `false` |

A driver's tier is not stored as a field in `DriverProfile`. The contract derives the tier from the driver's reputation score through `get_driver_tier`.

## 3. Storage Model

The contract defines the following storage key enum:

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

### Profile Storage Keys

| Key variant | Storage purpose |
|---|---|
| `DataKey::UserProfile(Address)` | Identifies a user profile using the user's address. |
| `DataKey::DriverProfile(Address)` | Identifies a driver profile using the driver's address. |

The address is part of the key, allowing each registered account to have its own profile entry.

### Persistent Storage

Both registration functions store profile records in Soroban persistent storage.

User profile storage:

```rust
let key = DataKey::UserProfile(user.clone());

env.storage().persistent().set(&key, &profile);
```

Driver profile storage:

```rust
let key = DataKey::DriverProfile(driver.clone());

env.storage().persistent().set(&key, &profile);
```

Persistent storage is used because profile records need to remain available beyond a single contract invocation.

### Storage TTL Extension

After storing a profile, the contract extends its persistent storage TTL:

```rust
env.storage().persistent().extend_ttl(&key, 518400, 518400);
```

This is applied to both user and driver profile entries.

The values are ledger-based TTL parameters. They should not be interpreted as seconds or calendar days.

## 4. Profile Retrieval

### get_user_profile

Function signature:

```rust
pub fn get_user_profile(env: Env, user: Address) -> UserProfile
```

The function:

1. Constructs `DataKey::UserProfile(user)`.
2. Reads the profile from persistent storage.
3. Returns the stored `UserProfile`.
4. Panics with `SwiftChainError::ProviderNotFound` when the profile does not exist.

### get_driver_profile

Function signature:

```rust
pub fn get_driver_profile(env: Env, driver: Address) -> DriverProfile
```

The function:

1. Constructs `DataKey::DriverProfile(driver)`.
2. Reads the profile from persistent storage.
3. Returns the stored `DriverProfile`.
4. Panics with `SwiftChainError::ProviderNotFound` when the profile does not exist.

Unlike the delivery contract's similarly named query, the identity/reputation contract's `get_driver_profile` does not construct a fallback profile.

## 5. Profile Creation and Update Behavior

### User Profiles

A user profile is created by `register_user`.

The function:

- Requires authorization from the user address.
- Creates the profile using the supplied address and current ledger timestamp.
- Rejects an existing profile key.
- Stores the profile in persistent storage.
- Extends the storage TTL.
- Publishes the `user_registered` event.

The user registration function does not update an existing profile.

### Driver Profiles

A driver profile is created by `register_driver`.

The function:

- Requires authorization from the driver address.
- Rejects an existing driver profile key.
- Initializes delivery count to zero.
- Initializes reputation score to 50.
- Sets KYC verification to false.
- Stores the profile in persistent storage.
- Extends the storage TTL.
- Publishes the `driver_registered` event.

Driver profile fields may subsequently be updated by contract functions such as KYC status management and reputation management.

## 6. Driver Tier Is Derived, Not Stored

The identity/reputation contract defines:

```rust
pub enum DriverTier {
    Bronze,
    Silver,
    Gold,
}
```

The tier is calculated by `get_driver_tier` using the stored `reputation_score`.

Current thresholds:

| Reputation score | Tier |
|---|---|
| Below `50` | Bronze |
| `50` to `74` | Silver |
| `75` and above | Gold |

There is no `tier` field in either `UserProfile` or `DriverProfile`.

Applications should query `get_driver_tier` when they need the driver's current tier instead of expecting a tier value to be stored in the profile.

## 7. Important Shared-Type Distinction

The repository also defines profile structures in:

```text
contracts/shared_types/lib.rs
```

The shared-types `DriverProfile` has the same five fields as the identity/reputation contract's `DriverProfile`.

However, the shared-types `UserProfile` is defined differently:

```rust
pub struct UserProfile {
    pub address: Address,
    pub registered_at: u64,
}
```

The identity/reputation contract's local `UserProfile` uses `join_date`, not `registered_at`.

These are separate Rust type definitions. Documentation and integrations should use the type belonging to the relevant contract rather than assuming the two `UserProfile` definitions are identical.

## 8. Source References

- `contracts/identity_reputation_contract/lib.rs`
- `contracts/identity_reputation_contract/test.rs`
- `contracts/shared_types/lib.rs`
- `docs/architecture/smart-contract-architecture.md`
