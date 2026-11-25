
# Escrow Refund Mechanism

## 1. Overview

The SwiftChain escrow contract provides a refund mechanism that returns locked delivery funds to the original sender when a delivery is cancelled or when a dispute is resolved in favor of the sender.

The refund operation is implemented by `refund_escrow` in:

`contracts/escrow_contract/lib.rs`

The escrow contract uses Soroban token interfaces to transfer the escrowed asset from the contract back to the original sender.

The refund process is designed to enforce caller authorization, validate the escrow state, verify available token balances, and persist the resulting escrow state.

## 2. Refund Eligibility

The `refund_escrow` function accepts the following parameters:

```rust
pub fn refund_escrow(
    env: Env,
    caller: Address,
    delivery_id: u64,
)
```

A refund is permitted only when all the following conditions are satisfied.

### 2.1 Authorized caller

The caller must be either:

- The original sender recorded in the escrow.
- The configured escrow administrator.

The caller must authenticate using:

```rust
caller.require_auth();
```

The authorization check compares the caller against the original sender and configured administrator.

Any other caller is rejected with:

`SwiftChainError::Unauthorized`

### 2.2 Valid escrow state

The escrow must have one of the following states:

- `EscrowStatus::Locked`
- `EscrowStatus::Paused`

The contract rejects refunds when the escrow is:

- `Released`
- `Refunded`

Any other unsupported state is rejected with:

`EscrowError::InvalidState`

This prevents an already released or refunded escrow from being refunded again.

### 2.3 Sufficient token balance

Before transferring funds, the contract retrieves its balance for the token associated with the escrow.

The balance must be greater than or equal to the recorded escrow amount.

If the balance is insufficient, the contract returns:

`EscrowError::InsufficientFunds`

The balance verification mechanism is documented separately in:

`docs/escrow_contract/balance-guard/balance-verification.md`

## 3. Refund Execution Flow

The refund process follows these steps:

1. The caller invokes `refund_escrow` with the delivery identifier.
2. The caller authenticates through Soroban authorization.
3. The contract loads the corresponding `EscrowRecord` from persistent storage.
4. The contract verifies that the caller is the original sender or administrator.
5. The contract verifies that the escrow status is `Locked` or `Paused`.
6. The contract checks the escrow token balance.
7. The contract transfers the recorded escrow amount to the original sender.
8. The escrow status is updated to `EscrowStatus::Refunded`.
9. The updated escrow record is persisted.
10. An `escrow_refunded` event is published.

## 4. Token Flow

The refund transfers the complete recorded escrow amount back to the original sender.

The token transfer is performed using:

```rust
token::Client::new(&env, &record.token).transfer(
    &env.current_contract_address(),
    &record.sender,
    &record.amount,
);
```

The transfer has three important properties:

- The source is the escrow contract address.
- The destination is the original sender's address.
- The amount is the full recorded escrow amount.

No platform fee is deducted by `refund_escrow`.

The refund therefore restores the original sender's recorded escrow principal, assuming the token transfer succeeds.

## 5. Escrow State Update

After the token transfer, the contract changes the escrow status:

```rust
record.status = EscrowStatus::Refunded;
```

The updated record is persisted through:

```rust
save_escrow(&env, delivery_id, &record);
```

The storage operation also extends the persistent storage time-to-live.

The resulting state prevents another refund through the normal refund function.

## 6. Refund Event

After the refund is successfully processed, the escrow contract publishes:

```rust
env.events().publish(
    (events::escrow_refunded(&env), delivery_id),
    (record.sender, record.amount),
);
```

The event contains:

| Component | Value |
|---|---|
| Event topic | `escrow_refunded` |
| Indexed topic value | `delivery_id` |
| Payload field 1 | Original sender address |
| Payload field 2 | Refunded amount |

The event allows an off-chain event consumer to identify the refunded delivery and update its corresponding read model.

## 7. Delivery Cancellation Integration

The delivery contract integrates cancellation with escrow refunds.

The relevant function is:

`cancel_delivery` in `contracts/delivery_contract/lib.rs`.

The cancellation process:

1. Requires authentication from the original delivery sender.
2. Loads the delivery record.
3. Verifies that the caller owns the delivery.
4. Validates the transition to `DeliveryStatus::Cancelled`.
5. Retrieves the configured escrow contract address.
6. Invokes `refund_escrow` using the sender address and delivery identifier.
7. Updates the delivery status to `Cancelled`.
8. Persists the delivery record.
9. Publishes the `delivery_cancelled` event.

The escrow refund must succeed before the delivery cancellation state is persisted.

Consequently, a failed refund prevents the cancellation operation from completing successfully.

### Important distinction

The current `refund_escrow` implementation does not independently query the delivery contract to verify a failed-delivery status or cancellation status.

Its direct eligibility checks are based on caller authorization, escrow state, and available token balance.

Delivery cancellation provides an integration path for sender-initiated refunds. Administrator-initiated refunds and paused-escrow refunds are also supported by the escrow contract.

## 8. Dispute Resolution Refunds

The escrow contract also supports refunds through dispute resolution.

The `resolve_dispute` function can refund the original sender when the administrator chooses the refund outcome.

The dispute must be in the `Paused` state.

The dispute resolution contract also exposes a separate sender-refund operation through:

`resolve_dispute_refund_sender`

in:

`contracts/dispute_resolution_contract/lib.rs`

These dispute-resolution operations should not be confused with the direct `refund_escrow` authorization rules. Their own administrator and dispute validation rules apply.

## 9. Reentrancy and Transaction Safety

The refund function performs several checks before transferring tokens:

- Caller authentication.
- Sender or administrator authorization.
- Escrow state validation.
- Token balance verification.

These checks prevent unauthorized callers from initiating refunds and prevent refunds from invalid escrow states.

### 9.1 No explicit reentrancy lock

The current `refund_escrow` implementation does not define or use a dedicated reentrancy-lock variable.

It must therefore not be described as implementing a custom checks-effects-interactions reentrancy guard.

### 9.2 External token interaction

The token transfer is an external contract interaction.

The contract updates the escrow state after the transfer succeeds.

Developers should preserve this distinction when extending the refund mechanism, particularly if additional external contract calls are introduced.

### 9.3 Transaction failure behavior

Soroban contract execution is transactional.

If a contract invocation fails, its uncommitted state changes and associated effects are not committed as a successful transaction.

Therefore, a failed token transfer must not result in a successfully persisted `Refunded` escrow state.

This transactional behavior complements, but does not replace, explicit authorization and state validation.

### 9.4 Future security considerations

Any future changes to the refund mechanism should:

- Preserve caller authorization.
- Prevent duplicate refunds.
- Preserve escrow state validation.
- Verify token balances before transfers.
- Avoid introducing unsafe external-call sequences.
- Include regression tests for failed transfers and unauthorized calls.
- Introduce an explicit reentrancy protection mechanism if required by future contract interactions and threat analysis.

## 10. Error Conditions

| Condition | Expected error |
|---|---|
| Caller is not authenticated | Soroban authorization failure |
| Caller is neither sender nor administrator | `SwiftChainError::Unauthorized` |
| Escrow record does not exist | `EscrowError::DeliveryNotFound` |
| Escrow is already released | `EscrowError::InvalidState` |
| Escrow is already refunded | `EscrowError::InvalidState` |
| Escrow balance is insufficient | `EscrowError::InsufficientFunds` |
| Token transfer fails | Transaction failure |

## 11. Related Source Files

- `contracts/escrow_contract/lib.rs`
- `contracts/escrow_contract/test.rs`
- `contracts/delivery_contract/lib.rs`
- `contracts/dispute_resolution_contract/lib.rs`
- `contracts/shared_types/lib.rs`

## 12. Summary

The SwiftChain refund mechanism returns the original escrow principal to the sender while enforcing authentication, caller authorization, valid escrow states, and sufficient token balances.

The delivery cancellation and dispute-resolution workflows integrate with this mechanism.

Refund-related events provide an observable record of successful refunds for future off-chain indexing and application updates.
