
# Mark Delivery as In Transit

## Overview

The `mark_in_transit` entry point allows the driver assigned to a delivery to record that the delivery has entered the transit stage.

It updates the delivery status from `Active` to `InTransit`, records the current ledger timestamp, extends the delivery record's persistent storage TTL, and publishes an event.

Implementation reference: `contracts/delivery_contract/lib.rs`.

## Function Signature

```rust
pub fn mark_in_transit(env: Env, driver: Address, delivery_id: DeliveryId)
```

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment used for storage, ledger timestamp, and event operations. |
| `driver` | `Address` | Address of the driver attempting to update the delivery. |
| `delivery_id` | `DeliveryId` | Identifier of the delivery being updated. |

## Authorization Requirements

The function first requires the supplied driver's authorization:

```rust
driver.require_auth();
```

It then loads the delivery record and checks whether the supplied driver matches the driver assigned to that delivery.

```rust
match &delivery.driver {
    Some(assigned) if *assigned == driver => {}
    _ => panic!("NotAuthorized"),
}
```

This means:

- The driver must authorize the transaction.
- A delivery must have an assigned driver.
- The supplied driver address must match the assigned driver address.
- An unassigned driver or a different driver cannot mark the delivery as in transit.

The function does not allow an arbitrary authenticated account to update the transit status.

## State Precondition

The delivery must currently be in the `Active` state.

The function enforces this through:

```rust
validate_transition(delivery.status.clone(), DeliveryStatus::InTransit)
    .unwrap_or_else(|_| panic!("InvalidState"));
```

The state machine permits:

```text
Active → InTransit
```

Other current states are rejected for this operation.

For example:

| Current status | Attempted status | Result |
|---|---|---|
| `Pending` | `InTransit` | Rejected |
| `Active` | `InTransit` | Allowed |
| `InTransit` | `InTransit` | Rejected |
| `Delivered` | `InTransit` | Rejected |
| `Disputed` | `InTransit` | Rejected |
| `Cancelled` | `InTransit` | Rejected |

## Execution Flow

The function performs these operations in order:

1. Requires authorization from the supplied driver address.
2. Builds the persistent storage key using `delivery_key(delivery_id)`.
3. Loads the delivery record.
4. Rejects the operation if the delivery record does not exist.
5. Verifies that the supplied driver is the assigned driver.
6. Validates the transition from `Active` to `InTransit`.
7. Reads the current ledger timestamp.
8. Updates the delivery status to `InTransit`.
9. Stores the timestamp in `transit_started_at`.
10. Saves the updated delivery record.
11. Extends the persistent storage TTL for the delivery record.
12. Publishes the `DeliveryInTransit` event.

## Timestamp Handling

The transit timestamp is obtained from the Soroban ledger:

```rust
let timestamp = env.ledger().timestamp();
```

It is stored in the delivery record:

```rust
delivery.transit_started_at = Some(timestamp);
```

The timestamp represents the ledger time at which the contract recorded the transit status update. It is not a timestamp supplied by the client.

## Storage Update

After updating the delivery record, the contract writes it to persistent storage and extends its TTL:

```rust
env.storage().persistent().set(&key, &delivery);
env.storage().persistent().extend_ttl(&key, 518400, 518400);
```

The TTL values are ledger-based values, not seconds or days.

## Event Emission

On successful execution, the contract publishes:

```rust
env.events().publish(
    (Symbol::new(&env, "DeliveryInTransit"),),
    (delivery_id, driver, timestamp),
);
```

| Event component | Value |
|---|---|
| Event topic | `DeliveryInTransit` |
| First event data item | `delivery_id` |
| Second event data item | `driver` |
| Third event data item | `timestamp` |

Applications and indexers can monitor this event to detect when a driver records a delivery as in transit.

The event is emitted only after the delivery record has been updated successfully.

## Failure Conditions

| Condition | Result |
|---|---|
| Driver does not authorize the operation | Authorization failure |
| Delivery ID does not identify an existing record | `DeliveryNotFound` panic path (`DeliveryNotFound` message) |
| Delivery has no assigned driver | `NotAuthorized` |
| Supplied driver is not the assigned driver | `NotAuthorized` |
| Delivery is not in `Active` state | `InvalidState` |

## Related Source Files

- `contracts/delivery_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `docs/architecture/smart-contract-architecture.md`

## Summary

`mark_in_transit` is a driver-only lifecycle operation. It requires the assigned driver's authorization, enforces the `Active → InTransit` transition, records the ledger timestamp, updates persistent storage, extends the record TTL, and emits `DeliveryInTransit`.
