
# Escrow Initialization and Query Functions

## Overview

The `escrow_contract` must be initialized before its protocol configuration and administrative functions can be used.

Initialization establishes the protocol administrator and stores the token and platform-fee configuration. The contract also exposes read-only functions for retrieving its administrator, configured token, protocol version, and a status value.

This document describes the implementation in:

- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`

## 1. Initialization

### Function signature

```rust
pub fn init(
    env: Env,
    admin: Address,
    token: Address,
    platform_fee_bps: u32,
)
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment supplied to the contract invocation. |
| `admin` | `Address` | Address stored as the protocol administrator. |
| `token` | `Address` | Token contract address stored as the protocol's configured token. |
| `platform_fee_bps` | `u32` | Initial platform fee expressed in basis points. |

One basis point represents 0.01%. For example, `250` basis points represents a 2.5% fee.

### Initialization behavior

The implementation performs the following operations:

1. Checks whether `StorageKey::Admin` already exists in instance storage.
2. Rejects initialization if an administrator has already been stored.
3. Stores the supplied administrator address.
4. Creates a `ProtocolConfig` value containing the token address, initial fee, and protocol version.
5. Stores the configuration under `StorageKey::ProtocolConfig`.
6. Publishes the `ProtocolInitialized` event.

The contract uses `SwiftChainError::AlreadyInitialized` when initialization has already occurred.

### Protocol version

The initial protocol version is obtained from:

```rust
pub const PROTOCOL_VERSION: u32 = 1;
```

The initialization function assigns this value to `ProtocolConfig.protocol_version`.

### Storage layout

The shared storage definitions are declared in `contracts/shared_types/lib.rs`.

| Storage key | Storage type | Stored value |
|---|---|---|
| `StorageKey::Admin` | Instance storage | Administrator `Address` |
| `StorageKey::ProtocolConfig` | Instance storage | `ProtocolConfig` |

The configuration structure is:

```rust
pub struct ProtocolConfig {
    pub token: Address,
    pub platform_fee_bps: u32,
    pub protocol_version: u32,
}
```

Conceptually, the initialized contract stores:

```text
Instance Storage
├── StorageKey::Admin
│   └── administrator address
│
└── StorageKey::ProtocolConfig
    ├── token
    ├── platform_fee_bps
    └── protocol_version
```

The administrator is stored separately from the protocol configuration.

### Important implementation notes

The current `init` implementation:

- Does not call `admin.require_auth()`.
- Does not validate the supplied token address beyond accepting it as an `Address`.
- Does not enforce the same maximum platform-fee limit used by `update_platform_fee`.
- Prevents a second initialization by checking whether the administrator storage entry already exists.

In particular, `update_platform_fee` rejects values above 1,000 basis points, but `init` does not currently perform that validation. Documentation should not imply that initialization enforces the same limit.

## 2. Query functions

The escrow contract exposes the following read-only functions.

### 2.1 `get_status`

**Function signature**

```rust
pub fn get_status(_env: Env) -> DeliveryStatus
```

**Return type:** `DeliveryStatus`

The function currently returns:

```rust
DeliveryStatus::Pending
```

It does not read an escrow record or accept a delivery identifier. Therefore, its result is a fixed status value rather than the current status of an individual escrow.

The `DeliveryStatus` enum is defined in `contracts/shared_types/lib.rs`:

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

**Important:** Do not use `get_status()` as an escrow-specific status lookup. For an individual escrow, use `get_escrow(delivery_id)`, which returns an `EscrowRecord` containing its actual `status`.

### 2.2 `get_admin`

**Function signature**

```rust
pub fn get_admin(env: Env) -> Address
```

**Return type:** `Address`

Returns the administrator address stored under:

```rust
StorageKey::Admin
```

If the administrator has not been initialized, the function fails with:

```rust
SwiftChainError::NotInitialized
```

### 2.3 `get_token`

**Function signature**

```rust
pub fn get_token(env: Env) -> Address
```

**Return type:** `Address`

Loads the protocol configuration and returns its configured token address:

```rust
load_protocol_config(&env).token
```

If the protocol configuration is unavailable, the function fails with:

```rust
SwiftChainError::NotInitialized
```

### 2.4 `get_protocol_version`

**Function signature**

```rust
pub fn get_protocol_version(env: Env) -> u32
```

**Return type:** `u32`

Loads the protocol configuration and returns:

```rust
load_protocol_config(&env).protocol_version
```

For a contract initialized using the current implementation, the returned value is:

```text
1
```

If the protocol configuration is unavailable, the function fails with:

```rust
SwiftChainError::NotInitialized
```

## 3. Example initialization flow

The following example follows the generated Soroban contract-client pattern used in the repository tests.

```rust
use soroban_sdk::{
    testutils::Address as _,
    Address,
};

let env = Env::default();
let contract_id = env.register(EscrowContract, ());
let client = EscrowContractClient::new(&env, &contract_id);

let admin = Address::generate(&env);
let token_admin = Address::generate(&env);

let token = env
    .register_stellar_asset_contract_v2(token_admin)
    .address();

// Initialize the escrow contract.
client.init(&admin, &token, &250);

// Read the initialized configuration.
let stored_admin = client.get_admin();
let stored_token = client.get_token();
let protocol_version = client.get_protocol_version();
let platform_fee = client.get_platform_fee();

assert_eq!(stored_admin, admin);
assert_eq!(stored_token, token);
assert_eq!(protocol_version, 1);
assert_eq!(platform_fee, 250);
```

This is an illustrative test/client flow. A deployed contract must first exist on the target Soroban network, and the invocation must be submitted through the appropriate client or transaction tooling.

## 4. Initialization event

After successful initialization, the contract publishes:

```rust
ProtocolInitialized {
    admin,
    token,
    platform_fee_bps,
    protocol_version,
}
```

The event topic is:

```text
ProtocolInitialized
```

Indexers can use this event to identify the initial administrator and protocol configuration.

## 5. Related functions

The following functions complement initialization and querying:

| Function | Purpose |
|---|---|
| `get_platform_fee()` | Returns the configured platform fee in basis points. |
| `update_platform_fee(admin, new_fee_bps)` | Allows the authorized administrator to update the fee. |
| `get_escrow(delivery_id)` | Returns an individual escrow record. |
| `get_settlement_contract()` | Returns the optional configured settlement contract address. |

## 6. Source references

- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `contracts/escrow_contract/test.rs`
- `docs/architecture/smart-contract-architecture.md`
