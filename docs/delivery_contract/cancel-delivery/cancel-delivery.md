
# Delivery Cancellation and Escrow Refund

## Overview

The delivery cancellation flow allows the original sender to cancel an eligible delivery. The delivery contract authenticates the sender, verifies ownership of the delivery, checks whether the current delivery state permits cancellation, and invokes the escrow contract's `refund_escrow` function.

The implementation is located in:

- `contracts/delivery_contract/lib.rs`
- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`

The relevant entry point is `DeliveryContract::cancel_delivery`.

## 1. Cancellation Function

```rust
pub fn cancel_delivery(
    env: Env,
    sender: Address,
    delivery_id: DeliveryId
)
```

Parameters:

| Parameter | Description |
|---|---|
| `env` | Soroban contract execution environment |
| `sender` | Address of the delivery sender |
| `delivery_id` | Identifier of the delivery to cancel |

## 2. Authorized Caller

The function first requires sender authorization:

```rust
sender.require_auth();
```

It loads the delivery record using the supplied delivery identifier. If the record does not exist, execution fails with `DeliveryNotFound`.

The contract then verifies that the supplied sender is the sender recorded in the delivery:

```rust
if delivery.sender != sender {
    panic!("NotAuthorized");
}
```

Consequently, the current delivery cancellation implementation authorizes only the original sender.

The driver and recipient are not authorized to call this cancellation path merely because they are associated with the delivery.

## 3. Cancellation State Rules

Before refunding escrow, the contract validates the transition to `Cancelled`:

```rust
validate_transition(
    delivery.status.clone(),
    DeliveryStatus::Cancelled
)
.unwrap_or_else(|_| panic!("InvalidState"));
```

The delivery contract currently permits these transitions:

| Current state | Target state | Cancellation |
|---|---|---|
| `Pending` | `Cancelled` | Allowed |
| `Active` | `Cancelled` | Allowed |
| `InTransit` | `Cancelled` | Not allowed |
| `Disputed` | `Cancelled` | Allowed |
| `Delivered` | `Cancelled` | Not allowed |
| `Cancelled` | `Cancelled` | Not allowed |

These rules are defined by `validate_transition` in `contracts/delivery_contract/lib.rs`.

A cancellation attempt from an unsupported state fails with `InvalidState` before the escrow refund invocation is reached.

## 4. Cross-Contract Escrow Refund

After authorization and state validation, the delivery contract retrieves the configured escrow contract address:

```rust
let escrow_address: Address = env
    .storage()
    .instance()
    .get(&DataKey::EscrowContract)
    .unwrap_or_else(|| panic!("EscrowNotConfigured"));
```

It then invokes:

```rust
use soroban_sdk::IntoVal;

let _: () = env.invoke_contract(
    &escrow_address,
    &soroban_sdk::Symbol::new(&env, "refund_escrow"),
    soroban_sdk::vec![
        &env,
        sender.into_val(&env),
        u64::from(delivery_id).into_val(&env),
    ],
);
```

The escrow function signature is:

```rust
pub fn refund_escrow(
    env: Env,
    caller: Address,
    delivery_id: u64
)
```

The delivery contract passes the original sender as the caller argument and converts the delivery identifier to `u64`.

### Escrow authorization

The escrow contract requires caller authorization:

```rust
caller.require_auth();
```

It then checks whether the caller is:

- The escrow administrator; or
- The sender recorded in the escrow record.

The sender must therefore match the escrow record's sender unless the caller is the administrator.

### Eligible escrow states

The escrow contract permits refunds when the escrow status is either:

- `EscrowStatus::Locked`
- `EscrowStatus::Paused`

Other escrow states are rejected with `EscrowError::InvalidState`.

This means the escrow contract can refund a paused escrow, which is relevant when a delivery has entered the disputed state.

### Refund operation

Before transferring tokens, the escrow contract checks that its token balance is sufficient.

On a successful refund:

1. The escrow contract transfers the full escrow amount to the recorded sender.
2. The escrow record is updated to `EscrowStatus::Refunded`.
3. The escrow record is saved and its TTL is extended.
4. An `escrow_refunded` event is published.

The refund path does not calculate or deduct the platform fee.

## 5. Delivery State Finalization

After the escrow refund call succeeds, the delivery contract finalizes cancellation:

```rust
delivery.status = DeliveryStatus::Cancelled;

env.storage().persistent().set(&key, &delivery);
env.storage().persistent().extend_ttl(&key, 518400, 518400);
```

The contract then publishes the `delivery_cancelled` event with the delivery identifier and sender address.

The order is important: the escrow refund is invoked before the delivery record is changed to `Cancelled`.

If the escrow invocation fails, execution does not reach the delivery-state update. The failed transaction is rolled back by Soroban transaction atomicity.

## 6. Cancellation Flow

```text
Original Sender
      |
      | Authorizes transaction
      v
DeliveryContract.cancel_delivery
      |
      | Load delivery record
      v
Verify sender matches delivery.sender
      |
      v
Validate transition to Cancelled
      |
      v
Invoke EscrowContract.refund_escrow
      |
      | Check caller authorization
      | Check Locked or Paused status
      | Check token balance
      | Transfer escrow amount to sender
      | Set escrow status to Refunded
      v
Return successfully to DeliveryContract
      |
      | Set delivery status to Cancelled
      | Save delivery record
      | Extend delivery record TTL
      v
Publish delivery_cancelled
```

## 7. Failure Conditions

| Condition | Expected behavior |
|---|---|
| Sender does not authorize | Transaction authorization fails |
| Delivery record does not exist | `DeliveryNotFound` |
| Caller is not the original sender | `NotAuthorized` |
| Current delivery state does not permit cancellation | `InvalidState` |
| Escrow contract is not configured | `EscrowNotConfigured` |
| Escrow caller is unauthorized | Escrow authorization error |
| Escrow is neither Locked nor Paused | Escrow `InvalidState` |
| Escrow token balance is insufficient | Escrow `InsufficientFunds` |

## 8. Related Events

The delivery contract publishes:

- `delivery_cancelled`

The escrow contract publishes:

- `escrow_refunded`

Off-chain applications can use these events to track cancellation and refund activity. The on-chain delivery and escrow records remain authoritative.

## 9. Architecture Reference

See:

- `docs/architecture/smart-contract-architecture.md`
- `contracts/delivery_contract/lib.rs`
- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`

## Summary

The current cancellation path is sender-authorized. It checks the delivery state, invokes the escrow contract's `refund_escrow` function, and marks the delivery as `Cancelled` only after the refund call succeeds. The escrow contract independently validates the sender, escrow status, and available token balance before refunding the sender.
