
# Escrow TTL and Storage Ledger Extension

## Overview

Soroban stores contract data in ledger entries that have a time-to-live (TTL).

If a storage entry is not extended before its TTL expires, it can become archived. An archived entry is not available through ordinary contract storage reads until it is restored according to Soroban's archival rules.

The escrow contract uses persistent storage for individual escrow records and instance storage for protocol-wide configuration.

To reduce the risk of escrow records becoming archived while they are still needed, the contract extends the TTL of escrow records when they are saved and loaded.

This document describes the implementation in:

- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`

## 1. TTL configuration

The escrow contract defines the following constants:

```rust
pub mod constants {
    pub const ESCROW_TTL_THRESHOLD: u32 = 518400;
    pub const ESCROW_TTL_EXTEND_TO: u32 = 518400;
    pub const PROTOCOL_VERSION: u32 = 1;
}
```

### TTL constants

| Constant | Value | Purpose |
|---|---:|---|
| `ESCROW_TTL_THRESHOLD` | `518400` | Remaining TTL threshold that triggers an extension. |
| `ESCROW_TTL_EXTEND_TO` | `518400` | Target TTL requested when the extension is applied. |

The values are measured in ledger entries/ledgers, not seconds.

The contract uses the Soroban storage API:

```rust
extend_ttl(key, threshold, extend_to)
```

An extension is requested when the entry's remaining TTL is below the configured threshold. The target is the configured extension value.

These constants should not be interpreted as a guaranteed wall-clock duration. Actual elapsed time depends on the network's ledger close time.

## 2. Storage types used by the escrow contract

The shared storage key enum is defined in `contracts/shared_types/lib.rs`:

```rust
pub enum StorageKey {
    Admin,
    Delivery(DeliveryId),
    Escrow(DeliveryId),
    DriverProfile(Address),
    ProtocolConfig,
}
```

The escrow contract uses two storage categories.

### 2.1 Instance storage

Instance storage contains protocol-wide contract information.

The escrow contract stores:

| Key | Stored value |
|---|---|
| `StorageKey::Admin` | Administrator address |
| `StorageKey::ProtocolConfig` | `ProtocolConfig` structure |
| `DataKey::PendingAdmin` | Proposed administrator address, when present |
| `DataKey::SettlementContract` | Configured settlement contract address, when present |

The `DataKey` enum is declared locally in `contracts/escrow_contract/lib.rs`:

```rust
enum DataKey {
    PendingAdmin,
    SettlementContract,
}
```

These values belong to the contract's instance storage lifetime rather than individual persistent escrow-record entries.

The contract calls:

```rust
env.storage().instance().extend_ttl(
    constants::ESCROW_TTL_THRESHOLD,
    constants::ESCROW_TTL_EXTEND_TO,
);
```

in the `propose_admin` and `accept_admin` functions.

This extends the contract instance entry's TTL, which protects the instance-storage data together.

### 2.2 Persistent storage

Individual escrow records are stored in persistent storage.

The key is constructed using:

```rust
escrow_key(delivery_id)
```

The shared helper returns:

```rust
StorageKey::Escrow(delivery_id)
```

The value stored under this key is an `EscrowRecord` containing:

- Sender address.
- Recipient address.
- Driver address.
- Token address.
- Escrow amount.
- Escrow status.
- Creation timestamp.
- Dispute reporter, when present.
- Dispute timestamp, when present.

Each delivery ID therefore has its own persistent escrow record.

## 3. Escrow record TTL extension

The contract centralizes persistent escrow storage in two helper functions:

- `save_escrow`
- `load_escrow`

### 3.1 `save_escrow`

Implementation:

```rust
fn save_escrow(
    env: &Env,
    delivery_id: u64,
    record: &EscrowRecord,
) {
    let key = escrow_key(delivery_id);

    env.storage().persistent().set(&key, record);

    env.storage().persistent().extend_ttl(
        &key,
        constants::ESCROW_TTL_THRESHOLD,
        constants::ESCROW_TTL_EXTEND_TO,
    );
}
```

The function:

1. Constructs the escrow storage key.
2. Writes the supplied record to persistent storage.
3. Requests a TTL extension for that persistent entry.

This means that updates to an escrow record also refresh its TTL.

### 3.2 `load_escrow`

Implementation:

```rust
fn load_escrow(
    env: &Env,
    delivery_id: u64,
) -> EscrowRecord {
    let key = escrow_key(delivery_id);

    let record: EscrowRecord = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| {
            panic_with_error!(
                env,
                EscrowError::DeliveryNotFound
            )
        });

    env.storage().persistent().extend_ttl(
        &key,
        constants::ESCROW_TTL_THRESHOLD,
        constants::ESCROW_TTL_EXTEND_TO,
    );

    record
}
```

The function:

1. Constructs the escrow key.
2. Reads the escrow record from persistent storage.
3. Rejects the operation if the record cannot be found.
4. Requests a TTL extension for the retrieved record.
5. Returns the record.

The missing-record path uses:

```rust
EscrowError::DeliveryNotFound
```

A failed lookup does not create a replacement escrow record.

## 4. Which operations refresh escrow TTL?

Because the lifecycle functions use `load_escrow` and/or `save_escrow`, their normal execution refreshes the relevant escrow entry's TTL.

| Operation | TTL behavior |
|---|---|
| `create_escrow` | Calls `save_escrow`, setting the record and requesting TTL extension. |
| `get_escrow` | Calls `load_escrow`, retrieving the record and requesting TTL extension. |
| `release_escrow` | Calls `load_escrow` and `save_escrow`. |
| `refund_escrow` | Calls `load_escrow` and `save_escrow`. |
| `raise_dispute` | Calls `load_escrow` and `save_escrow`. |
| `resolve_dispute` | Calls `load_escrow` and `save_escrow`. |
| `resolve_dispute_split` | Calls `load_escrow` and `save_escrow`. |
| `freeze_funds` | Calls `load_escrow` and, when it changes a locked record, `save_escrow`. |

The extension is performed for the individual escrow key being accessed or updated. It is not a bulk refresh of every escrow record.

## 5. Instance-storage TTL extension

The contract also extends the instance storage TTL in two administrator-management operations.

### `propose_admin`

After saving the pending administrator, the function calls:

```rust
env.storage().instance().extend_ttl(
    constants::ESCROW_TTL_THRESHOLD,
    constants::ESCROW_TTL_EXTEND_TO,
);
```

### `accept_admin`

After updating the administrator and removing the pending administrator entry, the function calls the same instance-storage extension.

### Important distinction

The persistent escrow-record extension and instance-storage extension operate on different storage categories.

| Storage category | Example data | Extension mechanism |
|---|---|---|
| Persistent storage | `StorageKey::Escrow(delivery_id)` | `persistent().extend_ttl(&key, threshold, extend_to)` |
| Instance storage | Admin and protocol configuration | `instance().extend_ttl(threshold, extend_to)` |

The current implementation does not explicitly call instance `extend_ttl` inside `init`, `get_admin`, `get_token`, or `get_protocol_version`.

## 6. Why TTL extension matters

Soroban storage entries are subject to network archival rules.

When an entry is not kept alive before its TTL expires, it may be archived. A contract operation that expects the archived data to be available cannot safely assume that an ordinary storage read will retrieve it.

For escrow, losing access to a record can affect the ability to:

- Retrieve the original sender and recipient.
- Verify the driver and token associated with a delivery.
- Determine whether funds are locked, paused, released, or refunded.
- Complete dispute resolution.
- Read the escrow amount and dispute metadata.

The contract therefore refreshes the relevant persistent escrow record when it is accessed or updated.

## 7. Operational limitations and archival considerations

TTL extension reduces archival risk, but the current implementation does not provide a background keep-alive process.

In particular:

1. Escrow records are refreshed when their corresponding contract functions successfully load or save them.
2. An escrow that is not accessed or updated does not receive periodic automatic refreshes from this contract.
3. The contract does not iterate through all escrow records to extend their TTL in bulk.
4. The `load_escrow` helper expects the persistent record to be readable; it does not itself implement an archival restoration workflow.
5. The instance-storage TTL is refreshed in the administrator proposal and acceptance functions, not on every read-only query.

For long-lived or inactive escrows, an operational keep-alive and archival recovery strategy may be needed to ensure that records remain accessible when required.

Any restoration or automated keep-alive mechanism should follow Soroban's supported archival and restoration procedures rather than assuming that an expired entry can be recovered by an ordinary contract read.

## 8. Storage lifecycle summary

```text
Escrow creation
      |
      v
Build StorageKey::Escrow(delivery_id)
      |
      v
Write EscrowRecord to persistent storage
      |
      v
Request persistent TTL extension
      |
      v
Escrow remains available while its entry is live
      |
      v
Subsequent load/save operations
      |
      v
Request TTL extension again
```

## 9. Source references

- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `contracts/escrow_contract/test.rs`
- `docs/architecture/smart-contract-architecture.md`
