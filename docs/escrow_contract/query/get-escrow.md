
# Get Escrow State Query Function

## Overview

The escrow contract exposes `get_escrow` to retrieve the stored state of a delivery escrow using its delivery identifier.

The function returns the complete `EscrowRecord` associated with the supplied delivery ID.

The contract stores escrow records in Soroban persistent storage.

**Implementation reference:** `contracts/escrow_contract/lib.rs`

**Data structure reference:** `contracts/shared_types/lib.rs`

## 1. Function Signature

The contract function is:

```rust
pub fn get_escrow(
    env: Env,
    delivery_id: u64,
) -> EscrowRecord
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment. |
| `delivery_id` | `u64` | Unique identifier associated with the escrow. |

### Return Type

```rust
EscrowRecord
```

The function returns the stored escrow record.

## 2. Implementation

The implementation is:

```rust
pub fn get_escrow(env: Env, delivery_id: u64) -> EscrowRecord {
    if !env
        .storage()
        .persistent()
        .has(&escrow_key(delivery_id))
    {
        panic_with_error!(&env, EscrowError::DeliveryNotFound);
    }

    load_escrow(&env, delivery_id)
}
```

The function performs the following operations:

1. Receives the delivery identifier.
2. Generates the corresponding escrow storage key using `escrow_key(delivery_id)`.
3. Checks whether the escrow exists in persistent storage.
4. Raises `EscrowError::DeliveryNotFound` if no record exists.
5. Loads and returns the stored escrow record.

The internal `load_escrow` helper also extends the persistent storage entry's time-to-live.

## 3. EscrowRecord Structure

The return structure is defined in:

`contracts/shared_types/lib.rs`

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

### Returned Fields

| Field | Type | Description |
|---|---|---|
| `sender` | `Address` | Address that funded the escrow. |
| `recipient` | `Address` | Recipient associated with the delivery. |
| `driver` | `Address` | Assigned driver receiving the payout. |
| `token` | `Address` | Token contract address used for the escrow. |
| `amount` | `i128` | Amount deposited into escrow. |
| `status` | `EscrowState` | Current escrow lifecycle state. |
| `created_at` | `u64` | Ledger timestamp when the escrow was created. |
| `disputed_by` | `Option<Address>` | Address that raised the dispute, if applicable. |
| `disputed_at` | `Option<u64>` | Ledger timestamp when the dispute was recorded, if applicable. |

## 4. Escrow Status Values

The `EscrowState` enum is defined in `contracts/shared_types/lib.rs`.

```rust
pub enum EscrowState {
    Locked,
    Released,
    Refunded,
    Paused,
}
```

| Status | Meaning |
|---|---|
| `Locked` | Funds are held in escrow and have not been released or refunded. |
| `Released` | Funds have been processed for payment to the driver. |
| `Refunded` | Funds have been returned to the original sender. |
| `Paused` | Escrow processing has been paused for dispute handling. |

`EscrowStatus` is a type alias for `EscrowState`.

## 5. Error Handling

The function raises:

```rust
EscrowError::DeliveryNotFound
```

when the supplied delivery identifier does not correspond to an existing escrow record.

The error is defined in `contracts/escrow_contract/lib.rs`.

The query does not require a caller address or explicit authentication argument.

## 6. On-Chain Read Path

The on-chain read path is:

```text
Client / Backend
       |
       v
Soroban RPC
       |
       v
Escrow Contract
       |
       v
get_escrow(delivery_id)
       |
       v
Persistent Storage
       |
       v
EscrowRecord
```

The Soroban contract is the authoritative source of the escrow state.

A client or backend can invoke the contract query through the Stellar/Soroban read mechanism and decode the returned `EscrowRecord`.

## 7. Off-Chain Indexed Read Path

The SwiftChain backend can maintain an indexed representation of escrow state in MongoDB.

The indexer is responsible for consuming relevant on-chain escrow events and maintaining the backend's read model.

The off-chain read flow is:

```text
Escrow Contract
       |
       v
Soroban Events
       |
       v
Backend Indexer
       |
       v
MongoDB Escrow Read Model
       |
       v
Backend Read API
       |
       v
Frontend / Client
```

Relevant escrow events include:

- `escrow_funded`
- `escrow_released`
- `escrow_refunded`
- `delivery_disputed`
- `dispute_resolved`

The indexed representation supports application-level reads without requiring every application request to invoke the Soroban contract directly.

### Important Consistency Consideration

MongoDB is an off-chain indexed representation, not the authoritative escrow state.

Because indexing is asynchronous, the MongoDB read model may temporarily lag behind the blockchain.

For operations that require authoritative or up-to-date escrow state, the application should verify the state directly against the Soroban contract.

The backend should also account for event processing retries and duplicate event delivery when maintaining indexed records.

**Implementation note:** The MongoDB collection schema, indexer implementation, and backend API routes are outside this smart contract repository. Their exact names and response formats should be documented in the SwiftChain backend repository rather than assumed here.

## 8. Storage and Lifecycle Considerations

Escrow records are stored using Soroban persistent storage.

The storage key is generated through:

```rust
escrow_key(delivery_id)
```

The escrow helper extends the entry's time-to-live using the contract's configured TTL constants.

This helps keep frequently accessed escrow records available on-chain.

## 9. Source References

- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `docs/architecture/smart-contract-architecture.md`
