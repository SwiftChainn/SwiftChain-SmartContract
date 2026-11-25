
# Recipient Delivery Confirmation and Escrow Release

## Overview

The delivery confirmation flow allows the intended recipient to confirm that a delivery has been completed. The delivery contract verifies the recipient's authorization and delivery state, invokes the escrow contract to release the escrowed funds, and finalizes the delivery record.

The implementation is located in:

- `contracts/delivery_contract/lib.rs`
- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`

The relevant entry point is `DeliveryContract::confirm_delivery`.

## 1. Confirmation Function

```rust
pub fn confirm_delivery(env: Env, recipient: Address, delivery_id: DeliveryId)
```

The function accepts:

| Parameter | Description |
|---|---|
| `env` | Soroban contract execution environment |
| `recipient` | Address of the recipient confirming delivery |
| `delivery_id` | Identifier of the delivery being confirmed |

## 2. Recipient Authorization

The first operation is:

```rust
recipient.require_auth();
```

This requires the recipient address to authorize the transaction.

The contract then loads the delivery record using its delivery identifier. If no record exists, execution fails with `DeliveryNotFound`.

The supplied recipient must also match the recipient stored in the delivery record:

```rust
if recipient != delivery.recipient {
    panic!("NotAuthorized");
}
```

This prevents another account from confirming a delivery on behalf of the intended recipient.

Both conditions are therefore required:

1. The supplied recipient must authorize the transaction.
2. The supplied recipient must match the recipient recorded for the delivery.

## 3. Delivery State Validation

Before invoking escrow, the contract validates the transition to `Delivered`:

```rust
validate_transition(
    delivery.status.clone(),
    DeliveryStatus::Delivered
)
.unwrap_or_else(|_| panic!("InvalidState"));
```

According to the delivery contract's state-transition rules, the following transitions to `Delivered` are accepted:

| Current state | Target state | Result |
|---|---|---|
| `InTransit` | `Delivered` | Allowed |
| `Disputed` | `Delivered` | Allowed by the delivery state validator |

Other states cannot transition directly to `Delivered`.

**Important escrow constraint:** Although the delivery state validator permits `Disputed -> Delivered`, the escrow contract's `release_escrow` function only accepts an escrow record in the `Locked` state. A disputed escrow is `Paused`. Therefore, confirmation of a disputed delivery cannot successfully release escrow through the current implementation unless the escrow state has first been resolved to an eligible state.

## 4. Cross-Contract Escrow Release

After authorization and delivery-state validation, the delivery contract retrieves the configured escrow contract address:

```rust
let escrow_address: Address = env
    .storage()
    .instance()
    .get(&DataKey::EscrowContract)
    .unwrap_or_else(|| panic!("EscrowNotConfigured"));
```

It then invokes the escrow contract:

```rust
use soroban_sdk::IntoVal;

let _: () = env.invoke_contract(
    &escrow_address,
    &soroban_sdk::Symbol::new(&env, "release_escrow"),
    soroban_sdk::vec![
        &env,
        recipient.into_val(&env),
        u64::from(delivery_id).into_val(&env),
    ],
);
```

The invocation passes:

- The recipient address as the escrow function's caller argument.
- The delivery identifier converted to `u64`.

The target function is:

```rust
pub fn release_escrow(
    env: Env,
    caller: Address,
    delivery_id: u64
)
```

### Escrow authorization

The escrow contract requires the supplied caller to authorize the invocation:

```rust
caller.require_auth();
```

It then checks whether the caller is either:

- The configured escrow administrator; or
- The recipient recorded in the escrow record.

The recipient must therefore match the escrow record's recipient unless the caller is the administrator.

### Escrow state and payout

The escrow contract requires the escrow record to have `EscrowStatus::Locked`.

It also checks that the escrow contract has sufficient token balance to cover the escrow amount.

On successful release:

1. The platform fee is calculated using the configured fee in basis points.
2. The remaining amount is paid to the driver.
3. If a settlement contract and a different driver-preferred asset are configured, the payout may be routed through the settlement contract.
4. The escrow record is updated to `EscrowStatus::Released`.
5. An escrow release event is published.

The escrow release is not simply a status update; it includes the corresponding token payout logic.

## 5. Delivery State Finalization

Only after the escrow invocation succeeds does the delivery contract update the delivery record:

```rust
delivery.status = DeliveryStatus::Delivered;
delivery.delivered_at = Some(env.ledger().timestamp());

env.storage().persistent().set(&key, &delivery);
env.storage().persistent().extend_ttl(&key, 518400, 518400);
```

This performs three operations:

- Changes the delivery status to `Delivered`.
- Records the ledger timestamp in `delivered_at`.
- Saves the updated record and extends its persistent-storage TTL.

If a driver is assigned, the delivery contract also updates the driver's local delivery profile:

```rust
profile.deliveries_completed += 1;
profile.reputation_score += 1;
```

The updated profile is saved and its TTL is extended.

Finally, the contract publishes the `delivery_confirmed` event with the delivery identifier and recipient address.

## 6. Confirmation Flow

```text
Recipient
   |
   | Authorizes transaction
   v
DeliveryContract.confirm_delivery
   |
   | Load delivery record
   v
Verify recipient identity
   |
   v
Validate transition to Delivered
   |
   v
Invoke EscrowContract.release_escrow
   |
   | Check escrow authorization
   | Check Locked status and token balance
   | Pay driver and platform fee
   | Set escrow status to Released
   v
Return successfully to DeliveryContract
   |
   | Set delivery status to Delivered
   | Set delivered_at timestamp
   | Update driver profile, when assigned
   | Extend relevant persistent TTLs
   v
Publish delivery_confirmed
```

## 7. Failure Conditions

| Condition | Expected behavior |
|---|---|
| Recipient does not authorize | Transaction authorization fails |
| Delivery record does not exist | `DeliveryNotFound` |
| Caller is not the recorded recipient | `NotAuthorized` |
| Delivery cannot transition to Delivered | `InvalidState` |
| Escrow contract is not configured | `EscrowNotConfigured` |
| Escrow caller is unauthorized | Escrow authorization error |
| Escrow is not in `Locked` state | Escrow `InvalidState` |
| Escrow has insufficient token balance | Escrow `InsufficientFunds` |

Because the escrow invocation occurs before the delivery record is finalized, a failed escrow invocation prevents the confirmation function from reaching its delivery-state update. Soroban transaction execution provides atomic rollback for failed contract invocations.

## 8. Related Events

The delivery contract publishes:

- `delivery_confirmed`

The escrow contract publishes:

- `escrow_released`

These events can help off-chain applications track delivery completion and escrow payout activity. Event consumers should still treat the on-chain contract state as authoritative.

## 9. Architecture Reference

See:

- `docs/architecture/smart-contract-architecture.md`
- `contracts/delivery_contract/lib.rs`
- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`

## Summary

Delivery confirmation is recipient-authorized. The delivery contract validates the delivery, calls `release_escrow` on the configured escrow contract, and finalizes the delivery record only after the escrow call succeeds. The escrow contract independently validates authorization, escrow status, and token balance before making the payout.
