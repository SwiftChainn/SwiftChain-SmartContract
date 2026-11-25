
# Escrow Dispute Trigger and Fund Freeze

## Overview

The escrow contract supports dispute handling by changing an escrow record from `EscrowStatus::Locked` to `EscrowStatus::Paused`.

The primary dispute entry point is:

```rust
raise_dispute(env, caller, delivery_id)
```

When a valid dispute is raised, the contract records the reporting party and dispute timestamp, persists the updated escrow record, and publishes a dispute event.

This document describes the implementation in:

- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`

## 1. Escrow states

The escrow state enum is defined in `contracts/shared_types/lib.rs`:

```rust
pub enum EscrowState {
    Locked,
    Released,
    Refunded,
    Paused,
}
```

The escrow contract also exposes this enum through the alias:

```rust
pub type EscrowStatus = EscrowState;
```

For dispute handling, the important transition is:

```text
Locked
  |
  | raise_dispute()
  v
Paused
```

A paused escrow remains recorded in persistent storage. The contract does not transfer the escrowed funds merely because a dispute is raised.

## 2. Raising a dispute

### Function signature

```rust
pub fn raise_dispute(
    env: Env,
    caller: Address,
    delivery_id: u64,
)
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment. |
| `caller` | `Address` | Address raising the dispute. |
| `delivery_id` | `u64` | Identifier of the escrow record being disputed. |

### Authorization requirements

The caller must authorize the transaction:

```rust
caller.require_auth();
```

The contract then checks whether the caller is one of the two permitted parties:

```rust
if caller != record.sender && caller != record.recipient {
    panic_with_error!(&env, SwiftChainError::Unauthorized);
}
```

Consequently:

- The original sender can raise a dispute.
- The recipient can raise a dispute.
- The driver cannot raise a dispute through this function.
- An unrelated address cannot raise a dispute.

The administrator does not receive a special bypass in `raise_dispute`; the caller must still be the sender or recipient.

### Escrow-state requirement

The function only accepts an escrow whose current status is:

```rust
EscrowStatus::Locked
```

If the escrow is already paused, released, or refunded, the function rejects the operation with:

```rust
EscrowError::InvalidState
```

### Dispute execution

After authorization and state validation, the contract:

1. Reads the current ledger timestamp.
2. Changes the escrow status to `Paused`.
3. Stores the caller in `disputed_by`.
4. Stores the timestamp in `disputed_at`.
5. Saves the updated escrow record.
6. Publishes the dispute event.

The relevant state update is:

```rust
record.status = EscrowStatus::Paused;
record.disputed_by = Some(caller.clone());
record.disputed_at = Some(timestamp);

save_escrow(&env, delivery_id, &record);
```

## 3. Funds freezing behavior

Raising a dispute pauses the escrow's normal release process.

The tokens remain held by the escrow contract. The dispute operation itself does not:

- Transfer funds to the sender.
- Transfer funds to the driver.
- Deduct the platform fee.
- Settle the dispute.
- Delete the escrow record.

Instead, it changes the stored escrow state so that subsequent operations can apply their own state checks.

The intended dispute lifecycle is:

```text
Escrow funded
     |
     v
   Locked
     |
     | Sender or recipient raises dispute
     v
   Paused
     |
     | Administrator resolves dispute
     v
Released / Refunded / Split settlement
```

The actual finalization behavior depends on the administrator's selected resolution function.

## 4. Effect on release and refund operations

### 4.1 Release is blocked while paused

The `release_escrow` function requires:

```rust
if record.status != EscrowStatus::Locked {
    panic_with_error!(&env, EscrowError::InvalidState);
}
```

Because `raise_dispute` changes the status to `Paused`, a normal release attempt cannot proceed.

This prevents the regular release path from paying the driver while the escrow is paused.

### 4.2 Important: refund is currently allowed while paused

The current `refund_escrow` implementation checks:

```rust
if record.status != EscrowStatus::Locked
    && record.status != EscrowStatus::Paused
{
    panic_with_error!(&env, EscrowError::InvalidState);
}
```

This means a refund is permitted when the escrow is either:

- `Locked`
- `Paused`

The refund function also requires the caller to be either the original sender or the administrator.

Therefore, the current contract does **not** completely block refunds during a dispute.

A sender or administrator can still refund a paused escrow through `refund_escrow`, provided the other authorization and balance checks pass.

**Implementation gap:** If the intended security requirement is that both release and refund must be blocked until an administrator resolves the dispute, the Rust condition in `refund_escrow` must be changed in a separate contract-logic change. This documentation does not silently claim that the current implementation already enforces that behavior.

## 5. Dispute event

After a dispute is recorded, the contract publishes:

```rust
env.events().publish(
    (events::delivery_disputed(&env), delivery_id),
    (caller, timestamp),
);
```

The event uses the `delivery_disputed` topic and includes:

| Event component | Value |
|---|---|
| Topic | `delivery_disputed` |
| Delivery identifier | `delivery_id` |
| Event data | `(caller, timestamp)` |

An off-chain indexer can use this event to identify the disputed escrow and the address that raised the dispute.

## 6. Escrow record fields

The dispute-related fields are part of `EscrowRecord`:

```rust
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

| Field | Purpose |
|---|---|
| `status` | Records the current escrow state. |
| `disputed_by` | Stores the sender or recipient who raised the dispute. |
| `disputed_at` | Stores the ledger timestamp at which the dispute was recorded. |

Both dispute fields are optional because an escrow may exist without ever entering a dispute.

## 7. Separate freeze function

The contract also exposes:

```rust
pub fn freeze_funds(env: Env, delivery_id: u64)
```

This function loads the escrow and changes its status to `Paused` only when its current status is `Locked`.

It also records the current timestamp in `disputed_at`.

However, the current implementation:

- Does not require caller authorization.
- Does not set `disputed_by`.
- Does not publish a dispute event.
- Does not change an escrow that is already outside the `Locked` state.

This function is distinct from `raise_dispute` and should not be described as having the same authorization or event behavior.

## 8. Storage and persistence

The escrow record is stored using the persistent storage key returned by:

```rust
escrow_key(delivery_id)
```

The `save_escrow` helper persists the updated record and extends its TTL.

The `load_escrow` helper retrieves the record and also extends its TTL.

This ensures that dispute-related metadata is saved with the escrow record rather than existing only during the transaction.

## 9. Source references

- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `contracts/escrow_contract/test.rs`
- `docs/architecture/smart-contract-architecture.md`
