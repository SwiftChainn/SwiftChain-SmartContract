
# Escrow Balance Verification Guard

## 1. Overview

The SwiftChain escrow contract uses balance verification to ensure that sufficient tokens are available before executing escrow payout and refund operations.

The balance verification mechanism protects the relationship between:

- The amount recorded in an escrow.
- The actual token balance held by the escrow contract.

The relevant implementation is located in:

`contracts/escrow_contract/lib.rs`

The balance guard is particularly important when releasing escrow funds, refunding senders, and resolving disputes through split payments.

## 2. Escrow Balance Invariant

The fundamental balance invariant is:

**The escrow contract must hold sufficient tokens to satisfy the amount required by an escrow operation before executing the corresponding token transfer.**

For a single escrow record, the contract verifies:

```text
contract_token_balance >= escrow_record.amount
```

Where:

- `contract_token_balance` is the escrow contract's current balance for the token associated with the escrow.
- `escrow_record.amount` is the recorded amount associated with the delivery.

The balance check ensures that an escrow operation does not intentionally proceed when the contract's available token balance is below the required recorded amount.

### Important distinction

The current implementation checks the contract's aggregate balance for the relevant token.

It does not maintain a separate token-balance ledger or independently segregated token balance for each delivery.

Therefore, the invariant is a sufficient-balance check for the requested operation, not proof that the contract's entire aggregate balance is exclusively attributable to one delivery.

## 3. Balance Verification Implementation

The balance check uses the Soroban token client.

```rust
let contract_balance =
    token::Client::new(&env, &record.token)
        .balance(&env.current_contract_address());

if contract_balance < record.amount {
    panic_with_error!(&env, EscrowError::InsufficientFunds);
}
```

### Verification steps

1. Retrieve the token address from the escrow record.
2. Create a Soroban token client for that asset.
3. Query the escrow contract's token balance.
4. Compare the balance with the recorded escrow amount.
5. Reject the operation if the balance is insufficient.
6. Continue with the requested token transfer if the check succeeds.

## 4. Operations That Use the Balance Guard

### 4.1 Release escrow

Function:

`release_escrow`

The release operation verifies that the escrow contract holds sufficient funds before calculating and distributing the driver payout and platform fee.

The relevant sequence is:

1. Authenticate the caller.
2. Load the escrow record.
3. Verify administrator or recipient authorization.
4. Confirm that the escrow is `Locked`.
5. Verify the contract token balance.
6. Calculate the platform fee.
7. Calculate the driver payout.
8. Execute the payout.
9. Persist the `Released` state.
10. Publish the `escrow_released` event.

### 4.2 Refund escrow

Function:

`refund_escrow`

The refund operation verifies the token balance before returning the full recorded escrow amount to the original sender.

The sequence is:

1. Authenticate the caller.
2. Load the escrow record.
3. Verify sender or administrator authorization.
4. Confirm that the escrow is `Locked` or `Paused`.
5. Verify the contract token balance.
6. Transfer the recorded amount to the sender.
7. Persist the `Refunded` state.
8. Publish the `escrow_refunded` event.

### 4.3 Split dispute resolution

Function:

`resolve_dispute_split`

The split resolution operation also checks that the contract holds sufficient funds before distributing the sender and driver shares.

The contract calculates:

```rust
let sender_amount =
    record.amount.saturating_mul(sender_share_bps as i128) / 10000;

let driver_amount =
    record.amount.saturating_sub(sender_amount);
```

It then transfers the respective amounts and persists the resulting escrow state.

## 5. State and Balance Consistency

The balance verification mechanism helps prevent state transitions from proceeding when the required token funds are unavailable.

The expected relationship is:

| Escrow operation | Required balance | Resulting state |
|---|---|---|
| Release | At least the recorded escrow amount | `Released` |
| Refund | At least the recorded escrow amount | `Refunded` |
| Split resolution | At least the recorded escrow amount | `Refunded` in the current implementation |

### State update sequence

The contract generally performs the required balance verification before the associated token transfers and saves the updated escrow record after the transfer operations.

For example, in `refund_escrow`:

```rust
token::Client::new(&env, &record.token).transfer(
    &env.current_contract_address(),
    &record.sender,
    &record.amount,
);

record.status = EscrowStatus::Refunded;

save_escrow(&env, delivery_id, &record);
```

This ordering ensures that the contract does not intentionally mark the escrow as refunded before executing the token transfer.

Soroban transactional execution provides an additional consistency guarantee: if the contract invocation fails, its uncommitted state changes are not committed as a successful operation.

## 6. Failure Behavior

When the contract balance is insufficient, the operation is rejected.

The contract raises:

`EscrowError::InsufficientFunds`

This prevents the operation from proceeding to the associated payout or refund transfer.

### Example

Assume:

```text
Recorded escrow amount:  1,000 tokens
Contract token balance:   700 tokens
```

The verification is:

```text
700 >= 1,000
```

The result is false.

The contract therefore rejects the operation with:

`EscrowError::InsufficientFunds`

The escrow should not be marked as successfully released or refunded as a result of this failed invocation.

### Other possible failure conditions

| Condition | Expected behavior |
|---|---|
| Escrow record does not exist | `EscrowError::DeliveryNotFound` |
| Invalid escrow state | `EscrowError::InvalidState` |
| Insufficient token balance | `EscrowError::InsufficientFunds` |
| Unauthorized caller | `SwiftChainError::Unauthorized` |
| Token transfer failure | Transaction failure |

## 7. Interaction with Platform Fees

When releasing escrow, the contract calculates the platform fee using:

```rust
fn calculate_fee(amount: i128, platform_fee_bps: u32) -> i128 {
    amount.saturating_mul(platform_fee_bps as i128) / 10_000
}
```

The driver amount is calculated as:

```rust
let driver_amount = record.amount.saturating_sub(platform_fee);
```

The resulting transfers distribute the recorded escrow amount between:

- The driver, after the applicable platform fee.
- The administrator, receiving the platform fee.

The initial balance verification checks the full recorded escrow amount rather than checking only the driver's net payout.

## 8. Important Implementation Limitations

The existing balance guard should not be interpreted as a complete accounting or solvency mechanism.

In particular:

- It checks the aggregate token balance of the escrow contract.
- It does not maintain a separate reserved-balance counter per delivery.
- It does not prove that the entire contract balance belongs to one specific escrow.
- It does not independently verify that all historical deposits and withdrawals reconcile.
- It does not prevent unrelated external transfers from affecting the aggregate token balance.

These distinctions are important when extending the escrow accounting architecture.

## 9. Security and Maintenance Considerations

Future modifications to escrow operations should preserve the following requirements:

1. Authenticate the caller before processing protected operations.
2. Load and validate the escrow record.
3. Verify the permitted escrow state.
4. Check the relevant token balance before transfers.
5. Preserve the recorded token and escrow amount.
6. Handle failed token transfers without committing an incorrect escrow state.
7. Persist the resulting state only after the required transfers succeed.
8. Emit completion events only after successful processing.
9. Add regression tests for insufficient balances and failed transfers.

Any changes to the accounting model should also consider how multiple escrows sharing the same token contract are reconciled.

## 10. Source References

- `contracts/escrow_contract/lib.rs`
- `contracts/escrow_contract/test.rs`
- `contracts/shared_types/lib.rs`
- `docs/architecture/smart-contract-architecture.md`

## 11. Summary

The escrow balance verification guard ensures that the contract checks its available token balance before executing important escrow payout and refund operations.

It helps maintain consistency between escrow records and token transfers by rejecting operations when the available balance is insufficient.

Together with authorization, escrow state validation, persistent storage, and Soroban transactional execution, the balance guard contributes to the security of SwiftChain's escrow lifecycle.
