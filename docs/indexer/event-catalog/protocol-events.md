
# SwiftChain Protocol Event Catalog

## 1. Overview

SwiftChain uses Soroban contract events to expose important state transitions and business operations across its smart contract system.

Events are published using:

```rust
env.events().publish(topics, data);
```

The protocol event catalog documents the events emitted by the six contract implementations and identifies the shared event types defined in the `shared_types` library.

The catalog also describes how an off-chain Node.js indexer can consume these events and maintain application read models.

### Contract modules covered

1. `delivery_contract`
2. `escrow_contract`
3. `dispute_resolution_contract`
4. `fleet_management_contract`
5. `identity_reputation_contract`
6. `settlement_contract`
7. `shared_types` (shared library, not an independently deployed contract)

## 2. Understanding Soroban Events

A Soroban event publication has two major components:

- Topics: identifiers and optional indexed values used to categorize and locate events.
- Data: the payload containing the event's associated information.

For example:

```rust
env.events().publish(
    (Symbol::new(&env, "escrow_refunded"), delivery_id),
    (sender, amount),
);
```

This produces:

| Component | Value |
|---|---|
| Topic 1 | `escrow_refunded` |
| Topic 2 | `delivery_id` |
| Payload | `(sender, amount)` |

The topic tuple and payload tuple must be decoded according to the actual event publication in the contract source.

Do not assume that shared event structs automatically define the payload of every published event. The `env.events().publish(...)` invocation is the authoritative source for the emitted shape.

## 3. Delivery Contract Events

Source:

`contracts/delivery_contract/lib.rs`

### 3.1 DeliveryContractInitialized

| Field | Value |
|---|---|
| Topic | `DeliveryContractInitialized` |
| Payload | `(admin, escrow_contract)` |

Published when the delivery contract is initialized.

### 3.2 delivery_created

| Field | Value |
|---|---|
| Topic | `delivery_created` |
| Payload | `(delivery_id, sender)` |

Published when a new delivery record is created.

### 3.3 delivery_cancelled

| Field | Value |
|---|---|
| Topic | `delivery_cancelled` |
| Payload | `(delivery_id, sender)` |

Published after the sender successfully cancels a delivery and its escrow refund invocation succeeds.

### 3.4 driver_assigned

| Field | Value |
|---|---|
| Topic | `driver_assigned` |
| Payload | `(delivery_id, driver)` |

Published when a driver is assigned to a delivery.

### 3.5 DeliveryInTransit

| Field | Value |
|---|---|
| Topic | `DeliveryInTransit` |
| Payload | `(delivery_id, driver, timestamp)` |

Published when the assigned driver marks a delivery as being in transit.

### 3.6 delivery_confirmed

| Field | Value |
|---|---|
| Topic | `delivery_confirmed` |
| Payload | `(delivery_id, recipient)` |

Published when the recipient confirms delivery.

The delivery confirmation process also invokes the escrow release operation.

### 3.7 delivery_disputed

| Field | Value |
|---|---|
| Topic | `delivery_disputed` |
| Payload | `(delivery_id, caller, timestamp)` |

Published when a delivery dispute is raised through the delivery contract.

## 4. Escrow Contract Events

Source:

`contracts/escrow_contract/lib.rs`

### 4.1 ProtocolInitialized

| Field | Value |
|---|---|
| Topic | `ProtocolInitialized` |
| Payload | `ProtocolInitialized` struct |

Payload fields:

- `admin`
- `token`
- `platform_fee_bps`
- `protocol_version`

### 4.2 FeeUpdated

| Field | Value |
|---|---|
| Topic | `FeeUpdated` |
| Payload | `FeeUpdated` struct |

Payload fields:

- `old_fee`
- `new_fee`

### 4.3 AdminTransferred

| Field | Value |
|---|---|
| Topic | `AdminTransferred` |
| Payload | `(old_admin, new_admin)` |

Published after the pending administrator accepts the transfer.

### 4.4 escrow_funded

| Field | Value |
|---|---|
| Topic 1 | `escrow_funded` |
| Topic 2 | `delivery_id` |
| Payload | `(sender, recipient, amount)` |

Published after the escrow is funded and its record is saved.

### 4.5 escrow_released

| Field | Value |
|---|---|
| Topic 1 | `escrow_released` |
| Topic 2 | `delivery_id` |
| Payload | `(driver, driver_amount, platform_fee)` |

Published after escrow funds are released.

### 4.6 escrow_refunded

| Field | Value |
|---|---|
| Topic 1 | `escrow_refunded` |
| Topic 2 | `delivery_id` |
| Payload | `(sender, amount)` |

Published after the original sender receives the refunded escrow amount.

### 4.7 delivery_disputed

| Field | Value |
|---|---|
| Topic 1 | `delivery_disputed` |
| Topic 2 | `delivery_id` |
| Payload | `(caller, timestamp)` |

Published when the escrow is paused due to a dispute.

### 4.8 dispute_resolved

| Field | Value |
|---|---|
| Topic 1 | `dispute_resolved` |
| Topic 2 | `delivery_id` |
| Payload | `(caller, caller)` |

This is the actual payload currently published by both `resolve_dispute` and `resolve_dispute_split`.

The duplicated caller values should not be interpreted as separate refund and payout outcome indicators. Consumers should use the dispute-resolution contract's outcome-specific events where the actual resolution outcome is required.

## 5. Dispute Resolution Contract Events

Source:

`contracts/dispute_resolution_contract/lib.rs`

### 5.1 dispute_raised

| Field | Value |
|---|---|
| Topic 1 | `dispute_raised` |
| Topic 2 | `delivery_id` |
| Payload | `(caller, delivery_id)` |

### 5.2 evidence_added

| Field | Value |
|---|---|
| Topic 1 | `evidence_added` |
| Topic 2 | `delivery_id` |
| Payload | `(caller, delivery_id, evidence_hash)` |

The evidence hash identifies the evidence reference recorded by the contract.

### 5.3 dispute_resolved_refund

| Field | Value |
|---|---|
| Topic 1 | `dispute_resolved_refund` |
| Topic 2 | `delivery_id` |
| Payload | `(caller, delivery_id, driver, DISPUTE_REPUTATION_PENALTY)` |

Published when a dispute is resolved through the sender-refund path.

### 5.4 dispute_resolved_split

| Field | Value |
|---|---|
| Topic 1 | `dispute_resolved_split` |
| Topic 2 | `delivery_id` |
| Payload | `(caller, delivery_id)` |

### 5.5 dispute_resolved_payout

| Field | Value |
|---|---|
| Topic 1 | `dispute_resolved_payout` |
| Topic 2 | `delivery_id` |
| Payload | `(caller, delivery_id)` |

## 6. Fleet Management Contract Events

Source:

`contracts/fleet_management_contract/lib.rs`

### 6.1 fleet_registered

| Field | Value |
|---|---|
| Topic | `fleet_registered` |
| Payload | `(fleet_id, owner, treasury)` |

### 6.2 fleet_treasury_updated

| Field | Value |
|---|---|
| Topic | `fleet_treasury_updated` |
| Payload | `(fleet_id, owner, treasury)` |

### 6.3 driver_invited

| Field | Value |
|---|---|
| Topic | `driver_invited` |
| Payload | `(fleet_id, driver)` |

### 6.4 invite_accepted

| Field | Value |
|---|---|
| Topic | `invite_accepted` |
| Payload | `(fleet_id, driver)` |

### 6.5 driver_removed

| Field | Value |
|---|---|
| Topic | `driver_removed` |
| Payload | `(fleet_id, driver)` |

## 7. Identity and Reputation Contract Events

Source:

`contracts/identity_reputation_contract/lib.rs`

### 7.1 kyc_status_updated

| Field | Value |
|---|---|
| Topic | `kyc_status_updated` |
| Payload | `(driver, kyc_verified)` |

### 7.2 reputation_increased

| Field | Value |
|---|---|
| Topic | `reputation_increased` |
| Payload | `(driver, delivery_id, points)` |

### 7.3 reputation_decreased

| Field | Value |
|---|---|
| Topic | `reputation_decreased` |
| Payload | `(driver, points)` |

## 8. Settlement Contract Events

Source:

`contracts/settlement_contract/lib.rs`

### 8.1 settlement_swap_executed

| Field | Value |
|---|---|
| Topic | `settlement_swap_executed` |
| Payload | `(caller, recipient, amount_in, net)` |

Published when a settlement swap is executed successfully.

## 9. Shared Event Types

Source:

`contracts/shared_types/lib.rs`

The shared-types library defines the following event-related structures:

| Struct | Fields |
|---|---|
| `DeliveryCreatedEvent` | `delivery_id`, `sender`, `amount` |
| `EscrowFundedEvent` | `delivery_id`, `sender`, `token`, `amount` |
| `DriverAssignedEvent` | `delivery_id`, `driver` |
| `DeliveryConfirmedEvent` | `delivery_id`, `recipient`, `timestamp` |
| `EscrowReleasedEvent` | `delivery_id`, `driver`, `amount`, `platform_fee` |
| `DeliveryDisputedEvent` | `delivery_id`, `reporter`, `timestamp` |
| `EscrowRefundedEvent` | `delivery_id`, `sender`, `amount` |

These structures provide reusable representations of protocol event data.

However, their existence does not mean every contract currently publishes those exact structures. The actual publication calls documented above remain authoritative.

## 10. Off-Chain Indexer Architecture

### 10.1 Indexer responsibility

An off-chain Node.js indexer can consume Soroban events and maintain application-facing read models.

Its responsibilities should include:

1. Connecting to the configured Stellar/Soroban RPC endpoint.
2. Identifying the deployed contract addresses.
3. Reading contract events from the selected ledger range.
4. Filtering events by contract and relevant topic.
5. Decoding topic and payload values.
6. Normalizing contract-specific data.
7. Updating the appropriate MongoDB read models.
8. Recording the last successfully processed ledger position.
9. Recovering from interruptions without creating duplicate records.

The Node.js indexer implementation is not present in this smart contract repository. Consequently, the steps above describe the expected integration architecture, not verified existing backend functions.

### 10.2 Recommended event processing flow

```text
Soroban Contracts
       |
       v
Stellar / Soroban RPC
       |
       v
Node.js Event Consumer
       |
       v
Event Filtering and Decoding
       |
       v
Event Normalization
       |
       v
MongoDB Read Models
       |
       v
Application API / Dashboard
```

### 10.3 Recommended processing sequence

**Step 1: Discover events**

Read events from the configured contract addresses using the supported Soroban RPC event retrieval interface.

**Step 2: Filter events**

Filter by:

- Contract address.
- Event topic.
- Ledger range.
- Relevant delivery or entity identifier.

**Step 3: Decode events**

Decode the topic tuple and payload according to the contract-specific publication format.

**Step 4: Normalize events**

Convert contract values into the application backend's expected representation.

Preserve the original contract address, transaction hash, ledger sequence, event index, and event topic when available.

**Step 5: Update read models**

Apply the normalized event to the relevant application projection.

**Step 6: Persist the checkpoint**

Record the last successfully processed ledger and event position.

**Step 7: Recover safely**

On restart, resume from the last committed checkpoint and prevent duplicate processing.

## 11. Recommended MongoDB Read-Model Mapping

The following table proposes read-model responsibilities for the off-chain application.

These are recommended logical mappings, not confirmed MongoDB collection names in the current repository.

| Event category | Events | Recommended read model |
|---|---|---|
| Delivery | `delivery_created`, `delivery_cancelled`, `driver_assigned`, `DeliveryInTransit`, `delivery_confirmed` | Delivery records |
| Escrow | `escrow_funded`, `escrow_released`, `escrow_refunded` | Escrow records and transaction history |
| Disputes | `dispute_raised`, `evidence_added`, `dispute_resolved_*` | Dispute records |
| Fleet | `fleet_registered`, `fleet_treasury_updated`, `driver_invited`, `invite_accepted`, `driver_removed` | Fleet and membership records |
| Identity | `kyc_status_updated` | Driver profiles |
| Reputation | `reputation_increased`, `reputation_decreased` | Driver reputation records |
| Settlement | `settlement_swap_executed` | Settlement transaction history |
| Protocol configuration | `ProtocolInitialized`, `FeeUpdated`, `AdminTransferred` | Protocol configuration history |

### 11.1 Delivery read model

Recommended fields:

- `deliveryId`
- `sender`
- `recipient`
- `driver`
- `status`
- `createdAt`
- `deliveredAt`
- `lastUpdatedLedger`

### 11.2 Escrow read model

Recommended fields:

- `deliveryId`
- `sender`
- `recipient`
- `driver`
- `token`
- `amount`
- `status`
- `refundedAmount`
- `lastUpdatedLedger`

### 11.3 Dispute read model

Recommended fields:

- `deliveryId`
- `raisedBy`
- `status`
- `evidenceHashes`
- `resolutionType`
- `lastUpdatedLedger`

### 11.4 Fleet read model

Recommended fields:

- `fleetId`
- `owner`
- `treasury`
- `activeDrivers`

### 11.5 Driver read model

Recommended fields:

- `address`
- `kycVerified`
- `reputationScore`
- `deliveriesCompleted`

### 11.6 Settlement read model

Recommended fields:

- `caller`
- `recipient`
- `amountIn`
- `netAmount`
- `transactionHash`
- `ledgerSequence`

## 12. Event Processing Reliability

An indexer should account for the following considerations.

### Duplicate event processing

Use a stable event identity based on the available chain event metadata, such as contract address, transaction hash, and event index.

Enforce idempotent database writes.

### Ledger checkpoints

Persist the last processed ledger and event position to support reliable recovery.

### Event ordering

Preserve ledger and event ordering when applying changes to state-dependent projections.

### Failed database writes

Do not advance the processing checkpoint until the corresponding database changes have been committed.

### Reconciliation

Where practical, periodically compare off-chain projections with authoritative on-chain contract state.

### Historical recovery

Support replaying a selected ledger range to rebuild read models after data loss or projection changes.

## 13. Important Integration Considerations

- Contract event topics are case-sensitive.
- Payload structures differ between contracts.
- Some events include the delivery identifier as a topic; others include it only in the payload.
- Shared event structures must not be mistaken for the actual payload of every publication.
- A dispute outcome should be derived from the outcome-specific dispute event, not inferred from the generic escrow `dispute_resolved` event.
- Off-chain projections must not be treated as the authoritative source of escrow balances or delivery ownership.
- MongoDB collection names and actual Node.js subscription implementations must be aligned with the separate backend repository.

## 14. Source References

- `contracts/delivery_contract/lib.rs`
- `contracts/escrow_contract/lib.rs`
- `contracts/dispute_resolution_contract/lib.rs`
- `contracts/fleet_management_contract/lib.rs`
- `contracts/identity_reputation_contract/lib.rs`
- `contracts/settlement_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `docs/architecture/smart-contract-architecture.md`

## 15. Summary

The SwiftChain event system exposes delivery, escrow, dispute, fleet, identity, reputation, and settlement operations.

The event catalog provides the contract-level event names, topic structures, and payload representations required for off-chain consumption.

A Node.js indexer can use these events to maintain MongoDB read models while preserving the smart contracts as the authoritative source of on-chain state.
