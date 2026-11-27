
# Escrow Events and Indexer Integration

## Overview

The SwiftChain smart contracts emit Soroban events to communicate important delivery and escrow operations to external applications.

These events allow backend services, blockchain indexers, and monitoring systems to observe on-chain activity without repeatedly querying contract storage.

The primary event definitions and topic helpers are maintained in:

`contracts/shared_types/lib.rs`

Escrow event emission is implemented in:

`contracts/escrow_contract/lib.rs`

Delivery event emission is implemented in:

`contracts/delivery_contract/lib.rs`

## 1. Soroban Event Structure

The contracts publish events using:

```rust
env.events().publish(topics, data);
```

An event contains:

- **Topics:** Identifiers used to classify and filter events.
- **Data:** The event payload containing the values associated with the operation.

For example, escrow funding uses:

```rust
env.events().publish(
    (events::escrow_funded(&env), delivery_id),
    (sender, recipient, amount),
);
```

The event contains:

| Component | Value |
|---|---|
| Topic 1 | `escrow_funded` |
| Topic 2 | `delivery_id` |
| Data | `(sender, recipient, amount)` |

The event topic and payload are separate. Indexers must decode both according to the actual contract event definition.

## 2. Escrow Contract Events

### 2.1 Escrow Funded

**Event topic:** `escrow_funded`

**Triggered by:** `create_escrow`

```rust
env.events().publish(
    (events::escrow_funded(&env), delivery_id),
    (sender, recipient, amount),
);
```

| Field | Value |
|---|---|
| Topic 1 | `escrow_funded` |
| Topic 2 | `delivery_id` |
| Payload 1 | `sender` |
| Payload 2 | `recipient` |
| Payload 3 | `amount` |

This event indicates that the sender's tokens have been transferred into the escrow contract and the escrow record has been created.

### 2.2 Escrow Released

**Event topic:** `escrow_released`

**Triggered by:** `release_escrow`

```rust
env.events().publish(
    (events::escrow_released(&env), delivery_id),
    (record.driver, driver_amount, platform_fee),
);
```

| Field | Value |
|---|---|
| Topic 1 | `escrow_released` |
| Topic 2 | `delivery_id` |
| Payload 1 | `driver` |
| Payload 2 | `driver_amount` |
| Payload 3 | `platform_fee` |

The event identifies the driver payout and the platform fee deducted from the original escrow amount.

### 2.3 Escrow Refunded

**Event topic:** `escrow_refunded`

**Triggered by:** `refund_escrow`

```rust
env.events().publish(
    (events::escrow_refunded(&env), delivery_id),
    (record.sender, record.amount),
);
```

| Field | Value |
|---|---|
| Topic 1 | `escrow_refunded` |
| Topic 2 | `delivery_id` |
| Payload 1 | `sender` |
| Payload 2 | `amount` |

This event indicates that escrow funds have been returned to the original sender.

### 2.4 Delivery Disputed

**Event topic:** `delivery_disputed`

**Triggered by:** `raise_dispute`

```rust
env.events().publish(
    (events::delivery_disputed(&env), delivery_id),
    (caller, timestamp),
);
```

| Field | Value |
|---|---|
| Topic 1 | `delivery_disputed` |
| Topic 2 | `delivery_id` |
| Payload 1 | `caller` |
| Payload 2 | `timestamp` |

This event indicates that the escrow has transitioned from `Locked` to `Paused`.

### 2.5 Dispute Resolved

**Event topic:** `dispute_resolved`

**Triggered by:**
- `resolve_dispute`
- `resolve_dispute_split`

```rust
env.events().publish(
    (events::dispute_resolved(&env), delivery_id),
    (caller.clone(), caller),
);
```

| Field | Value |
|---|---|
| Topic 1 | `dispute_resolved` |
| Topic 2 | `delivery_id` |
| Payload 1 | `caller` |
| Payload 2 | `caller` |

**Important implementation detail:** Both payload positions currently contain the admin caller address. The event does not currently distinguish the selected resolution outcome, sender payout, driver payout, or split amounts.

An indexer must not interpret these two values as separate participants or as proof of a particular resolution outcome.

## 3. Delivery Contract Events

The delivery contract also emits events that describe delivery lifecycle operations.

These events are important because delivery state and escrow state are managed by separate contracts.

| Event | Triggering function | Topic | Payload |
|---|---|---|---|
| `delivery_created` | `create_delivery` | `delivery_created` | `(delivery_id, sender)` |
| `delivery_cancelled` | `cancel_delivery` | `delivery_cancelled` | `(delivery_id, sender)` |
| `driver_assigned` | `assign_driver` | `driver_assigned` | `(delivery_id, driver)` |
| `DeliveryInTransit` | `mark_in_transit` | `DeliveryInTransit` | `(delivery_id, driver, timestamp)` |
| `delivery_confirmed` | `confirm_delivery` | `delivery_confirmed` | `(delivery_id, recipient)` |
| `delivery_disputed` | `raise_dispute` | `delivery_disputed` | `(delivery_id, caller, timestamp)` |

Note that event topic casing is significant. The current implementation uses `DeliveryInTransit` with uppercase letters, unlike the lowercase naming convention of several other delivery events.

## 4. Platform and Administrative Events

The escrow contract also emits events that are not part of the ordinary escrow payment lifecycle.

### ProtocolInitialized

**Trigger:** `init`

Topic:

`ProtocolInitialized`

Payload:

```rust
ProtocolInitialized {
    admin,
    token,
    platform_fee_bps,
    protocol_version,
}
```

### FeeUpdated

**Trigger:** `update_platform_fee`

Topic:

`FeeUpdated`

Payload:

```rust
FeeUpdated {
    old_fee,
    new_fee: new_fee_bps,
}
```

The payload contains the previous and updated fee values in basis points.

### AdminTransferred

**Trigger:** `accept_admin`

Topic:

`AdminTransferred`

Payload:

```rust
(old_admin, new_admin)
```

This event is emitted only after the proposed administrator successfully accepts ownership.

`propose_admin` does not currently emit an event.

## 5. State Transition and Event Coverage

The following table documents the actual event coverage in the current implementation.

| Operation | Contract | State change | Event emitted |
|---|---|---|---|
| Create delivery | Delivery | New delivery record | `delivery_created` |
| Cancel delivery | Delivery | Delivery → Cancelled | `delivery_cancelled` |
| Assign driver | Delivery | Pending → Active | `driver_assigned` |
| Mark in transit | Delivery | Active → InTransit | `DeliveryInTransit` |
| Confirm delivery | Delivery | InTransit/Disputed → Delivered | `delivery_confirmed` |
| Raise dispute | Delivery | → Disputed | `delivery_disputed` |
| Create escrow | Escrow | New record → Locked | `escrow_funded` |
| Release escrow | Escrow | Locked → Released | `escrow_released` |
| Refund escrow | Escrow | Locked/Paused → Refunded | `escrow_refunded` |
| Raise escrow dispute | Escrow | Locked → Paused | `delivery_disputed` |
| Resolve dispute | Escrow | Paused → Released/Refunded | `dispute_resolved` |
| Resolve dispute split | Escrow | Paused → Refunded | `dispute_resolved` |
| Freeze funds | Escrow | Locked → Paused | No event |
| Propose admin | Escrow | Pending admin recorded | No event |
| Accept admin | Escrow | Admin ownership changed | `AdminTransferred` |
| Update platform fee | Escrow | Fee configuration updated | `FeeUpdated` |

### Event coverage considerations

The issue requires documentation of event-emission requirements for state transitions.

The table above describes existing behavior, not a claim that every state transition already has a dedicated event.

In particular:

- `freeze_funds` changes the escrow state without publishing an event.
- `propose_admin` stores a pending administrator without publishing an event.
- `resolve_dispute` and `resolve_dispute_split` share the same event payload, which does not communicate the actual resolution outcome.
- Delivery and escrow contracts can emit separate events for related operations. For example, a dispute raised through the delivery contract results in an escrow event followed by a delivery event.

These are implementation gaps worth tracking separately if strict one-event-per-transition guarantees are required.

## 6. How Indexers Should Consume Events

An indexer should process Soroban events using their contract address, ledger position, transaction hash, topics, and decoded event data.

### Recommended processing flow

1. Subscribe to or poll the Stellar Soroban event source.
2. Retrieve events from the configured contract addresses.
3. Identify the emitting contract.
4. Read the event topics.
5. Match the event name against supported event types.
6. Decode the payload using the corresponding event schema.
7. Extract the delivery identifier where available.
8. Update the corresponding off-chain delivery or escrow record.
9. Persist transaction and ledger references for traceability.
10. Handle duplicate events and retry temporary processing failures.

### Recommended indexer mapping

| Event | Suggested off-chain action |
|---|---|
| `delivery_created` | Create or update the delivery blockchain reference |
| `escrow_funded` | Mark payment as locked after validating the transaction |
| `driver_assigned` | Update the delivery's assigned driver |
| `DeliveryInTransit` | Update the delivery tracking status |
| `delivery_confirmed` | Record recipient confirmation |
| `delivery_disputed` | Mark the related delivery or escrow as disputed |
| `escrow_released` | Record driver payout and platform fee |
| `escrow_refunded` | Record the sender refund |
| `dispute_resolved` | Refresh the escrow record to determine the actual outcome |
| `FeeUpdated` | Refresh platform fee configuration |
| `AdminTransferred` | Refresh the contract administrator |

## 7. Reliability and Security Considerations

- Use transaction hashes and event positions to prevent duplicate processing.
- Do not assume events from different contracts have identical payload structures.
- Preserve the emitting contract address when storing indexed events.
- Validate event data before updating application records.
- Do not mark a payment as successful merely because a frontend initiated a transaction.
- Reconcile indexed information against on-chain contract state where necessary.
- Account for missing events and incomplete event coverage in monitoring systems.
- Treat events as evidence of on-chain execution, while keeping application-level processing idempotent.

## Source References

- `contracts/escrow_contract/lib.rs`
- `contracts/delivery_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `contracts/escrow_contract/test.rs`
- `docs/architecture/smart-contract-architecture.md`
