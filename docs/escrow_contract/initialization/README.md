# Escrow Contract — Initialization and Query Functions

This document covers the `init` function, the protocol-config storage layout, and the four
read-only query functions exposed by `EscrowContract`.

**Source file:** `contracts/escrow_contract/lib.rs`  
**Shared types:** `contracts/shared_types/lib.rs`

---

## Table of Contents

1. [Overview](#1-overview)
2. [init](#2-init)
3. [Storage Layout](#3-storage-layout)
4. [Query Functions](#4-query-functions)
   - [get_admin](#41-get_admin)
   - [get_token](#42-get_token)
   - [get_protocol_version](#43-get_protocol_version)
   - [get_status](#44-get_status)
5. [Error Reference](#5-error-reference)
6. [Initialization and Query Flow](#6-initialization-and-query-flow)

---

## 1. Overview

`EscrowContract` is a Soroban smart contract that holds funds in escrow for SwiftChain
logistics deliveries. Before any escrow operation can run, the contract must be initialized
exactly once via `init`. Calling `init` writes the admin address and the protocol
configuration into instance storage and emits a `ProtocolInitialized` event.

After initialization the query functions — `get_admin`, `get_token`,
`get_protocol_version`, and `get_status` — expose protocol state to callers and
off-chain tooling. All four functions are read-only and require no authorization.

---

## 2. `init`

### Signature

```rust
pub fn init(env: Env, admin: Address, token: Address, platform_fee_bps: u32)
```

### Parameters

| Parameter          | Type      | Description                                                                                                      |
|--------------------|-----------|------------------------------------------------------------------------------------------------------------------|
| `env`              | `Env`     | Soroban host environment injected by the runtime.                                                               |
| `admin`            | `Address` | Protocol administrator. This address can update the platform fee, propose admin transfers, and release disputes. |
| `token`            | `Address` | Stellar asset contract address that the escrow contract accepts for all escrow operations.                       |
| `platform_fee_bps` | `u32`     | Platform fee in basis points (1 bps = 0.01 %). A value of `250` means 2.5 %. Maximum enforced by `update_platform_fee` is `1000` (10 %). |

### Behavior

1. Checks whether `StorageKey::Admin` already exists in instance storage.
   If it does, the function panics with `SwiftChainError::AlreadyInitialized` (code 2).
2. Writes `admin` to `StorageKey::Admin` in instance storage.
3. Constructs a `ProtocolConfig` struct with `token`, `platform_fee_bps`, and
   `PROTOCOL_VERSION` (constant `1`), then writes it to `StorageKey::ProtocolConfig` in
   instance storage.
4. Emits a `ProtocolInitialized` event (see [Storage Layout](#3-storage-layout) for event shape).

`init` accepts **no authorization check on the `admin` address** — any caller can invoke
it once, but only once. After the first successful call the contract is permanently locked
against re-initialization.

### Example — Stellar CLI

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <DEPLOYER_SECRET_KEY> \
  --network testnet \
  -- init \
  --admin <ADMIN_ADDRESS> \
  --token <TOKEN_CONTRACT_ADDRESS> \
  --platform_fee_bps 250
```

Replace angle-bracket placeholders with real values. Never embed secret keys in scripts
committed to version control; use environment variables or a secrets manager instead.

---

## 3. Storage Layout

The contract uses two storage scopes: **instance** (tied to the contract instance TTL) and
**persistent** (per-delivery escrow records with their own TTL). This section covers the
instance keys written during initialization.

### Instance Storage Keys

| Key                          | Rust type         | Written by       | Description                                                   |
|------------------------------|-------------------|------------------|---------------------------------------------------------------|
| `StorageKey::Admin`          | `Address`         | `init`           | Protocol administrator address.                               |
| `StorageKey::ProtocolConfig` | `ProtocolConfig`  | `init`, `update_platform_fee` | Protocol-wide configuration struct (see below).  |
| `DataKey::PendingAdmin`      | `Address`         | `propose_admin`  | Candidate address in a two-step admin transfer. Present only while a transfer is in progress. |
| `DataKey::SettlementContract`| `Address`         | `set_settlement_contract` | Optional settlement contract used for cross-border FX routing on payout. Present only after `set_settlement_contract` is called. |

### `ProtocolConfig` Struct

Defined in `contracts/shared_types/lib.rs`:

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolConfig {
    pub token: Address,
    pub platform_fee_bps: u32,
    pub protocol_version: u32,
}
```

| Field                | Type      | Set during `init` to                        |
|----------------------|-----------|---------------------------------------------|
| `token`              | `Address` | The `token` argument passed to `init`.      |
| `platform_fee_bps`   | `u32`     | The `platform_fee_bps` argument passed to `init`. |
| `protocol_version`   | `u32`     | `constants::PROTOCOL_VERSION` = `1`.        |

### `ProtocolInitialized` Event

Emitted by `init` on successful completion.

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolInitialized {
    pub admin: Address,
    pub token: Address,
    pub platform_fee_bps: u32,
    pub protocol_version: u32,
}
```

| Field | Description |
|---|---|
| **Topics** | `(Symbol("ProtocolInitialized"),)` — a one-element tuple |
| **Data** | `ProtocolInitialized { admin, token, platform_fee_bps, protocol_version }` |

Off-chain indexers can subscribe to the `ProtocolInitialized` topic to detect and record
the initialization parameters of a newly deployed contract.

### Persistent Storage Keys (Escrow Records)

Each escrow is stored under a persistent key derived from the delivery identifier:

```rust
// From shared_types/lib.rs
pub fn escrow_key(id: impl Into<DeliveryId>) -> StorageKey {
    StorageKey::Escrow(id.into())
}
```

| Key                              | Type           | Description                                         |
|----------------------------------|----------------|-----------------------------------------------------|
| `StorageKey::Escrow(DeliveryId)` | `EscrowRecord` | Full escrow record for a single delivery. See `contracts/shared_types/lib.rs` for the `EscrowRecord` struct. |

Persistent entries are created by `create_escrow` and are outside the scope of initialization.

---

## 4. Query Functions

All four functions are read-only. They do not modify state and require no authorization.
Any address or off-chain client may call them.

### 4.1 `get_admin`

#### Signature

```rust
pub fn get_admin(env: Env) -> Address
```

#### Purpose

Returns the current protocol administrator address stored at `StorageKey::Admin`.

#### Return Type

`Address` — the admin address set during `init`.

#### Errors

| Condition              | Error                                 |
|------------------------|---------------------------------------|
| Contract not initialized | `SwiftChainError::NotInitialized` (code 3) |

#### Example — Stellar CLI

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <ANY_ACCOUNT> \
  --network testnet \
  -- get_admin
```

---

### 4.2 `get_token`

#### Signature

```rust
pub fn get_token(env: Env) -> Address
```

#### Purpose

Returns the token contract address stored in `ProtocolConfig.token`. This is the Stellar
asset contract that the escrow uses for all fund locking and payout operations.

#### Return Type

`Address` — the token address set during `init`.

#### Errors

| Condition              | Error                                 |
|------------------------|---------------------------------------|
| Contract not initialized | `SwiftChainError::NotInitialized` (code 3) |

#### Example — Stellar CLI

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <ANY_ACCOUNT> \
  --network testnet \
  -- get_token
```

---

### 4.3 `get_protocol_version`

#### Signature

```rust
pub fn get_protocol_version(env: Env) -> u32
```

#### Purpose

Returns the protocol version number stored in `ProtocolConfig.protocol_version`. The
current deployed version is `1` (defined by `constants::PROTOCOL_VERSION`). Off-chain
clients can use this to detect incompatible contract versions before interacting with the
contract.

#### Return Type

`u32` — always `1` for the current contract version.

#### Errors

| Condition              | Error                                 |
|------------------------|---------------------------------------|
| Contract not initialized | `SwiftChainError::NotInitialized` (code 3) |

#### Example — Stellar CLI

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <ANY_ACCOUNT> \
  --network testnet \
  -- get_protocol_version
```

---

### 4.4 `get_status`

#### Signature

```rust
pub fn get_status(_env: Env) -> DeliveryStatus
```

#### Purpose

Returns the protocol-level delivery status. This function is a static indicator and always
returns `DeliveryStatus::Pending` regardless of any on-chain state. It does not read
storage and does not require the contract to be initialized.

#### Return Type

`DeliveryStatus` — always `DeliveryStatus::Pending`.

`DeliveryStatus` is defined in `contracts/shared_types/lib.rs`:

```rust
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum DeliveryStatus {
    Pending,
    Active,
    InTransit,
    Delivered,
    Disputed,
    Cancelled,
}
```

#### Errors

None. This function never panics.

#### Example — Stellar CLI

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <ANY_ACCOUNT> \
  --network testnet \
  -- get_status
```

---

## 5. Error Reference

The errors that can arise during initialization and queries come from two enums.

### `SwiftChainError` (defined in `shared_types`)

| Variant              | Code | Triggered by                             |
|----------------------|------|------------------------------------------|
| `Unauthorized`       | 1    | Caller is not the stored admin.          |
| `AlreadyInitialized` | 2    | `init` called on an already-initialized contract. |
| `NotInitialized`     | 3    | Any query or admin-only function called before `init`. |

### `EscrowError` (defined in `escrow_contract`)

| Variant           | Code | Triggered by                                          |
|-------------------|------|-------------------------------------------------------|
| `InvalidState`    | 1    | Escrow operation attempted in a disallowed state.     |
| `DeliveryNotFound`| 2    | No escrow record found for the given `delivery_id`.   |
| `InsufficientFunds`| 3   | Contract balance is too low to complete the payout.   |
| `DuplicateDelivery`| 4   | `create_escrow` called with an already-registered ID. |
| `InvalidFee`      | 5    | `update_platform_fee` called with a value above 1000 bps. |

---

## 6. Initialization and Query Flow

The sequence below shows a complete deployment: compile, deploy, initialize, and
verify the stored configuration with the query functions.

### Step 1 — Build and deploy

```bash
# Build the escrow contract WASM
cargo build -p escrow_contract --target wasm32-unknown-unknown --release

# Generate a deployer identity (one-time setup)
stellar keys generate deployer --network testnet

# Deploy and capture the contract ID
CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/escrow_contract.wasm \
  --source deployer \
  --network testnet)

echo "Deployed contract: $CONTRACT_ID"
```

### Step 2 — Initialize

```bash
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source deployer \
  --network testnet \
  -- init \
  --admin "$ADMIN_ADDRESS" \
  --token "$TOKEN_CONTRACT_ADDRESS" \
  --platform_fee_bps 250
```

On success the contract emits a `ProtocolInitialized` event with `protocol_version = 1`.

A second call to `init` on the same contract will return error code `2`
(`AlreadyInitialized`).

### Step 3 — Verify stored configuration

```bash
# Confirm admin
stellar contract invoke \
  --id "$CONTRACT_ID" --source deployer --network testnet \
  -- get_admin

# Confirm token
stellar contract invoke \
  --id "$CONTRACT_ID" --source deployer --network testnet \
  -- get_token

# Confirm protocol version
stellar contract invoke \
  --id "$CONTRACT_ID" --source deployer --network testnet \
  -- get_protocol_version

# Confirm static status
stellar contract invoke \
  --id "$CONTRACT_ID" --source deployer --network testnet \
  -- get_status
```

Expected outputs:

| Function               | Expected output                                    |
|------------------------|----------------------------------------------------|
| `get_admin`            | The `ADMIN_ADDRESS` passed to `init`.              |
| `get_token`            | The `TOKEN_CONTRACT_ADDRESS` passed to `init`.     |
| `get_protocol_version` | `1`                                                |
| `get_status`           | `"Pending"`                                        |

### Step 4 — Inspect the emitted event (optional)

```bash
stellar events \
  --id "$CONTRACT_ID" \
  --network testnet \
  --topic "ProtocolInitialized"
```

The event data payload matches the `ProtocolInitialized` struct:

```json
{
  "admin": "<ADMIN_ADDRESS>",
  "token": "<TOKEN_CONTRACT_ADDRESS>",
  "platform_fee_bps": 250,
  "protocol_version": 1
}
```

---

## Related Documentation

- [Smart Contract Architecture](../../architecture/smart-contract-architecture.md) — system-level overview of all SwiftChain contracts.
- `contracts/escrow_contract/lib.rs` — full contract source.
- `contracts/shared_types/lib.rs` — shared structs, enums, and error definitions.
- `contracts/escrow_contract/test.rs` — unit and integration tests covering `init` and query functions.
