
# Delivery State Machine and Transition Enforcement

## Overview

The delivery contract uses a state machine to control the lifecycle of each delivery. The state machine prevents invalid status changes and ensures that lifecycle operations follow the transitions explicitly permitted by `validate_transition`.

The implementation is located in:

- `contracts/delivery_contract/lib.rs`
- `contracts/shared_types/lib.rs`

The shared `DeliveryStatus` enum defines the available states, while `validate_transition` determines whether a requested transition is allowed.

## DeliveryStatus Enum

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

| Status | Meaning |
|---|---|
| `Pending` | Delivery has been created but has not yet been activated through driver assignment. |
| `Active` | Delivery has been activated and assigned to a driver. |
| `InTransit` | The assigned driver has marked the delivery as in transit. |
| `Delivered` | Delivery has been confirmed by the recipient. |
| `Disputed` | The sender or recipient has raised a dispute. |
| `Cancelled` | Delivery has been cancelled by the sender through the cancellation flow. |

## Allowed Transition Table

The current implementation of `validate_transition` permits the following transitions:

| Current status | Allowed next status | Operation or purpose |
|---|---|---|
| `Pending` | `Active` | Driver assignment activates the delivery. |
| `Pending` | `Cancelled` | Sender cancellation. |
| `Active` | `InTransit` | Assigned driver starts transit. |
| `Active` | `Disputed` | Sender or recipient raises a dispute. |
| `Active` | `Cancelled` | Sender cancellation. |
| `InTransit` | `Delivered` | Recipient confirms delivery. |
| `InTransit` | `Disputed` | Sender or recipient raises a dispute. |
| `Disputed` | `Delivered` | Delivery confirmation, subject to the escrow release checks. |
| `Disputed` | `Cancelled` | Sender cancellation, subject to the escrow refund checks. |

All transitions not listed in this table are rejected by the current validator.

### Terminal states

`Delivered` and `Cancelled` are terminal states in the delivery state machine.

Neither state has an outgoing transition in `validate_transition`.

## State Machine Diagram

```text
                 ┌──────────────┐
                 │   Pending    │
                 └──────┬───────┘
                        │
             ┌──────────┴──────────┐
             │                     │
             ▼                     ▼
       ┌──────────┐          ┌───────────┐
       │  Active  │          │ Cancelled │
       └────┬─────┘          └───────────┘
            │
     ┌──────┼───────────────┐
     │      │               │
     ▼      ▼               ▼
┌───────────┐  ┌──────────┐  ┌───────────┐
│ InTransit │  │ Disputed │  │ Cancelled │
└─────┬─────┘  └────┬─────┘  └───────────┘
      │             │
  ┌───┴────┐    ┌───┴──────────┐
  │        │    │              │
  ▼        ▼    ▼              ▼
┌─────────┐ ┌──────────┐  ┌───────────┐
│Delivered│ │ Disputed │  │ Cancelled │
└─────────┘ └────┬─────┘  └───────────┘
                 │
                 ▼
            ┌──────────┐
            │Delivered │
            └──────────┘
```

The diagram illustrates the permitted delivery-status transitions. It does not represent every escrow status or guarantee that an escrow operation will succeed.

## validate_transition

The validator is defined in `contracts/delivery_contract/lib.rs`:

```rust
pub fn validate_transition(
    from: DeliveryStatus,
    to: DeliveryStatus,
) -> Result<(), DeliveryError>
```

It receives the current status and the requested next status.

The function matches the pair against the allowed transition list:

```rust
let valid = match (from, to) {
    (DeliveryStatus::Pending, DeliveryStatus::Active) => true,
    (DeliveryStatus::Pending, DeliveryStatus::Cancelled) => true,
    (DeliveryStatus::Active, DeliveryStatus::InTransit) => true,
    (DeliveryStatus::Active, DeliveryStatus::Disputed) => true,
    (DeliveryStatus::Active, DeliveryStatus::Cancelled) => true,
    (DeliveryStatus::InTransit, DeliveryStatus::Delivered) => true,
    (DeliveryStatus::InTransit, DeliveryStatus::Disputed) => true,
    (DeliveryStatus::Disputed, DeliveryStatus::Delivered) => true,
    (DeliveryStatus::Disputed, DeliveryStatus::Cancelled) => true,
    _ => false,
};
```

If the requested transition is allowed, the function returns:

```rust
Ok(())
```

Otherwise, it returns:

```rust
Err(DeliveryError::InvalidState)
```

## Error Handling

The delivery contract defines:

```rust
pub enum DeliveryError {
    InvalidState = 1,
}
```

Lifecycle functions use the validator and convert a rejected transition into an `InvalidState` panic path:

```rust
validate_transition(delivery.status.clone(), DeliveryStatus::InTransit)
    .unwrap_or_else(|_| panic!("InvalidState"));
```

The exact operation determines the requested destination status.

An invalid transition is rejected before the delivery record is updated.

## Lifecycle Operations That Use the Validator

| Function | Requested transition | Additional authorization or checks |
|---|---|---|
| `assign_driver` | Current state → `Active` | Caller must be the admin or the driver being assigned. |
| `mark_in_transit` | Current state → `InTransit` | Caller must be the assigned driver. |
| `confirm_delivery` | Current state → `Delivered` | Caller must be the recorded recipient; escrow release must succeed. |
| `cancel_delivery` | Current state → `Cancelled` | Caller must be the recorded sender; escrow refund must succeed. |
| `raise_dispute` | Current state → `Disputed` | Caller must be the recorded sender or recipient; escrow dispute call must succeed. |

The validator controls state transitions. It does not replace the authorization checks or escrow checks performed by these functions.

## Relationship Between Delivery and Escrow States

The delivery contract and escrow contract maintain separate state records.

The shared delivery states are:

- `Pending`
- `Active`
- `InTransit`
- `Delivered`
- `Disputed`
- `Cancelled`

The escrow contract uses `EscrowStatus` (an alias of `EscrowState`) with these states:

- `Locked`
- `Released`
- `Refunded`
- `Paused`

A delivery transition that passes `validate_transition` can still fail if the corresponding escrow operation rejects its own state or authorization requirements.

For example:

- `confirm_delivery` permits `Disputed → Delivered` at the delivery state-machine level, but `release_escrow` requires the escrow record to be `Locked`. A disputed escrow is normally `Paused`, so the escrow release check can reject that operation.
- `cancel_delivery` permits `Disputed → Cancelled`, and `refund_escrow` accepts escrow records in either `Locked` or `Paused` states.

The delivery record is updated only after the relevant cross-contract escrow call succeeds.

Escrow dispute resolution is managed by escrow entry points such as `resolve_dispute` and `resolve_dispute_split`. Those functions update escrow state; they do not automatically update the corresponding delivery record.

## Important Invariants

1. A delivery must exist before a lifecycle operation can update it.
2. Only transitions listed in `validate_transition` are accepted.
3. Authorization is checked separately from transition validity.
4. The delivery record is not updated when a required escrow call fails.
5. `Delivered` and `Cancelled` do not have outgoing transitions in the current validator.
6. Delivery and escrow statuses are separate and must not be treated as interchangeable.

## Related Source Files

- `contracts/delivery_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `contracts/escrow_contract/lib.rs`
- `docs/architecture/smart-contract-architecture.md`

## Summary

The delivery state machine enforces a defined set of lifecycle transitions through `validate_transition`. It rejects invalid status changes with `DeliveryError::InvalidState`, while authorization and escrow operations provide additional checks. The state machine's actual implementation—not a simplified linear workflow—is the source of truth for permitted transitions.
