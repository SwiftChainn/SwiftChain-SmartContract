
# Delivery Dispute Trigger and Escrow Pause

## Overview

The delivery dispute flow allows the original sender or intended recipient to raise a dispute against an eligible delivery.

The delivery contract verifies the caller, validates the delivery state, and invokes the escrow contract's `raise_dispute` function. The escrow contract then changes the escrow status from `Locked` to `Paused`, preventing normal escrow release or refund operations until the dispute is resolved through the appropriate escrow workflow.

Relevant implementation:

- `contracts/delivery_contract/lib.rs`
- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`

The delivery entry point is `DeliveryContract::raise_dispute`.

## 1. Dispute Function

```rust
pub fn raise_dispute(
    env: Env,
    caller: Address,
    delivery_id: DeliveryId
)
```

Parameters:

| Parameter | Description |
|---|---|
| `env` | Soroban contract execution environment |
| `caller` | Address of the account raising the dispute |
| `delivery_id` | Identifier of the delivery being disputed |

## 2. Caller Authorization

The function first requires the caller to authorize the transaction:

```rust
caller.require_auth();
```

It then loads the delivery record. If the record does not exist, execution fails with `DeliveryNotFound`.

The contract checks whether the caller is the original sender or the intended recipient:

```rust
let is_sender = caller == delivery.sender;
let is_recipient = caller == delivery.recipient;

if !is_sender && !is_recipient {
    panic!("NotAuthorized");
}
```

The current implementation therefore permits:

- The original sender.
- The intended recipient.

The assigned driver and unrelated accounts cannot raise a dispute through this function unless they are also the recorded sender or recipient.

## 3. Delivery State Validation

The delivery contract validates whether the delivery can transition to `Disputed`:

```rust
validate_transition(
    delivery.status.clone(),
    DeliveryStatus::Disputed
)
.unwrap_or_else(|_| panic!("InvalidState"));
```

The current state validator permits these transitions:

| Current state | Target state | Dispute |
|---|---|---|
| `Pending` | `Disputed` | Not allowed |
| `Active` | `Disputed` | Allowed |
| `InTransit` | `Disputed` | Allowed |
| `Disputed` | `Disputed` | Not allowed |
| `Delivered` | `Disputed` | Not allowed |
| `Cancelled` | `Disputed` | Not allowed |

A dispute cannot be raised against a delivery that is still `Pending` under the current state-transition rules.

## 4. Cross-Contract Escrow Pause

After authorization and state validation, the delivery contract retrieves the configured escrow contract address:

```rust
let escrow_address: Address = env
    .storage()
    .instance()
    .get(&DataKey::EscrowContract)
    .unwrap_or_else(|| panic!("EscrowNotConfigured"));
```

It invokes the escrow contract's `raise_dispute` function:

```rust
use soroban_sdk::IntoVal;

let _: () = env.invoke_contract(
    &escrow_address,
    &Symbol::new(&env, "raise_dispute"),
    soroban_sdk::vec![
        &env,
        caller.into_val(&env),
        u64::from(delivery_id).into_val(&env),
    ],
);
```

The escrow function signature is:

```rust
pub fn raise_dispute(
    env: Env,
    caller: Address,
    delivery_id: u64
)
```

The delivery contract passes the dispute initiator and delivery identifier to the escrow contract.

### Escrow authorization

The escrow contract requires caller authorization and loads the corresponding escrow record.

It independently checks that the caller is either:

- The sender recorded in the escrow record; or
- The recipient recorded in the escrow record.

This means the delivery and escrow contracts both enforce the sender-or-recipient restriction.

### Escrow state validation

The escrow contract accepts a dispute only when the escrow status is:

```rust
EscrowStatus::Locked
```

If the escrow is already released, refunded, or paused, the dispute invocation fails with `EscrowError::InvalidState`.

### Escrow freeze behavior

When the dispute is accepted, the escrow contract:

```rust
record.status = EscrowStatus::Paused;
record.disputed_by = Some(caller.clone());
record.disputed_at = Some(timestamp);
```

It then saves the escrow record, extends its TTL, and publishes a dispute event.

The `Paused` status prevents the normal `release_escrow` path, which requires `Locked`, from proceeding.

The escrow refund path does accept `Paused`, but it remains subject to the refund function's authorization checks.

## 5. Delivery State Update

The delivery contract invokes escrow before changing the delivery record.

After the escrow call succeeds, it records the dispute:

```rust
let timestamp = env.ledger().timestamp();

delivery.status = DeliveryStatus::Disputed;

env.storage().persistent().set(&key, &delivery);
env.storage().persistent().extend_ttl(&key, 518400, 518400);
```

The delivery contract then publishes:

```rust
env.events().publish(
    (Symbol::new(&env, "delivery_disputed"),),
    (delivery_id, caller, timestamp),
);
```

The event includes:

- The delivery identifier.
- The address that raised the dispute.
- The ledger timestamp.

## 6. Dispute Flow

```text
Sender or Recipient
        |
        | Authorizes transaction
        v
DeliveryContract.raise_dispute
        |
        | Load delivery record
        v
Verify caller is sender or recipient
        |
        v
Validate transition to Disputed
        |
        v
Invoke EscrowContract.raise_dispute
        |
        | Verify caller against escrow record
        | Require escrow status Locked
        | Set escrow status Paused
        | Record disputed_by and disputed_at
        | Save escrow and extend TTL
        v
Return successfully to DeliveryContract
        |
        | Set delivery status Disputed
        | Save delivery and extend TTL
        v
Publish delivery_disputed
```

## 7. Failure and Atomicity

The delivery contract deliberately invokes escrow before updating the delivery record.

This ordering ensures that the delivery record is not changed to `Disputed` if the escrow dispute call fails.

Possible failure conditions include:

| Condition | Expected behavior |
|---|---|
| Caller does not authorize | Transaction authorization fails |
| Delivery record does not exist | `DeliveryNotFound` |
| Caller is neither sender nor recipient | `NotAuthorized` |
| Delivery cannot transition to Disputed | `InvalidState` |
| Escrow contract is not configured | `EscrowNotConfigured` |
| Escrow caller does not match sender or recipient | Escrow authorization error |
| Escrow status is not Locked | Escrow `InvalidState` |

Soroban transaction execution is atomic. If the cross-contract invocation fails and the error propagates, the transaction does not leave the delivery record updated while the escrow operation has failed.

## 8. Dispute Resolution

The escrow contract exposes dispute-resolution operations, including:

```rust
pub fn resolve_dispute(
    env: Env,
    caller: Address,
    delivery_id: u64,
    release_to_driver: bool
)
```

The caller must be authorized as the escrow administrator.

The function requires the escrow status to be `Paused`.

Depending on the resolution decision, the escrow may:

- Release the funds to the driver, subject to the configured fee and payout logic; or
- Refund the sender.

The escrow record is then updated to `Released` or `Refunded`.

The escrow contract also provides a split-resolution operation for distributing funds according to a sender share in basis points.

**Important distinction:** The dispute trigger changes the delivery status to `Disputed` and the escrow status to `Paused`. The resolution functions operate on the escrow record. Do not assume that resolving escrow automatically changes the delivery record from `Disputed` to another delivery state; that behavior is not implemented by the dispute-trigger flow documented here.

## 9. Related Events

The delivery contract publishes:

- `delivery_disputed`

The escrow contract publishes:

- `delivery_disputed` through its event helper.

Both events relate to the dispute, but they originate from separate contracts and should be interpreted in their respective contract contexts.

## 10. Architecture Reference

See:

- `docs/architecture/smart-contract-architecture.md`
- `contracts/delivery_contract/lib.rs`
- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`

## Summary

The dispute trigger is available to the delivery sender and recipient. The delivery contract validates the delivery state and invokes the escrow contract before updating its own record. The escrow contract independently validates the caller and requires a `Locked` escrow before changing its status to `Paused`. This prevents the normal escrow release flow from proceeding while the dispute remains unresolved.
