
# Admin Dispute Resolution and Escrow Settlement

## Overview

The escrow contract provides two administrator-controlled functions for resolving disputes:

1. `resolve_dispute` — releases the escrow to the driver or refunds it to the original sender.
2. `resolve_dispute_split` — distributes the escrow between the original sender and the driver using a proportional basis-point allocation.

Both functions require administrator authorization and only operate on an escrow currently in the `Paused` state.

This document describes the implementation in:

- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`

## 1. Dispute resolution requirements

Before an escrow can be resolved through either function, it must have been placed in the paused state.

The expected transition is:

```text
Locked
  |
  | raise_dispute()
  v
Paused
  |
  | Administrator resolves dispute
  |
  +----------------------------+
  |                            |
  v                            v
resolve_dispute()       resolve_dispute_split()
  |                            |
  v                            v
Released / Refunded          Refunded
```

The administrator must authorize the transaction and must match the administrator address stored in the contract.

## 2. Administrator authorization

Both resolution functions execute:

```rust
caller.require_auth();
require_admin(&env, &caller);
```

The first operation requires authorization from the supplied caller address.

The second operation compares the supplied address with the administrator stored under:

```rust
StorageKey::Admin
```

If the caller is not the stored administrator, the contract rejects the operation with:

```rust
SwiftChainError::Unauthorized
```

A sender, recipient, or driver cannot independently execute these administrator-only resolution functions unless that address is also the configured administrator.

## 3. Single-outcome resolution

### Function signature

```rust
pub fn resolve_dispute(
    env: Env,
    caller: Address,
    delivery_id: u64,
    release_to_driver: bool,
)
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment. |
| `caller` | `Address` | Administrator authorizing the resolution. |
| `delivery_id` | `u64` | Identifier of the disputed escrow. |
| `release_to_driver` | `bool` | Selects whether the driver receives the escrow payout or the sender receives a refund. |

### Preconditions

The function:

1. Requires administrator authorization.
2. Loads the escrow record using `delivery_id`.
3. Checks that the escrow status is `EscrowStatus::Paused`.

If the escrow is not paused, the function rejects the operation with:

```rust
EscrowError::InvalidState
```

### 3.1 Release funds to the driver

When:

```rust
release_to_driver == true
```

the contract calculates the platform fee using the configured platform fee in basis points.

The calculation is:

```rust
let platform_fee = calculate_fee(
    record.amount,
    platform_fee_bps,
);

let driver_amount = record.amount.saturating_sub(platform_fee);
```

The fee calculation uses:

```rust
amount.saturating_mul(platform_fee_bps as i128) / 10_000
```

The contract then:

1. Pays the driver through `payout_driver`.
2. Transfers the platform fee to the configured administrator when the fee is greater than zero.
3. Changes the escrow status to `Released`.
4. Saves the updated escrow record.

The resulting distribution is:

```text
Original escrow amount
        |
        +----------------------+
        |                      |
        v                      v
   Driver payout          Platform fee
   amount - fee           transferred to admin
```

If a settlement contract has been configured and the driver has a preferred asset, `payout_driver` may invoke the settlement contract to perform the configured asset swap. Otherwise, the driver receives the escrow token directly.

### 3.2 Refund funds to the sender

When:

```rust
release_to_driver == false
```

the contract transfers the full escrow amount to the original sender:

```rust
token::Client::new(&env, &record.token).transfer(
    &env.current_contract_address(),
    &record.sender,
    &record.amount,
);
```

The contract then changes the status to:

```rust
EscrowStatus::Refunded
```

The full escrow amount is returned to the sender. No platform fee is deducted in this refund branch.

### 3.3 Finalization

After either outcome, the contract persists the updated escrow record:

```rust
save_escrow(&env, delivery_id, &record);
```

The resulting states are:

| Resolution choice | Recipient of funds | Final escrow status |
|---|---|---|
| `release_to_driver = true` | Driver, less platform fee; administrator receives fee when applicable | `Released` |
| `release_to_driver = false` | Original sender receives the full amount | `Refunded` |

## 4. Proportional split resolution

### Function signature

```rust
pub fn resolve_dispute_split(
    env: Env,
    caller: Address,
    delivery_id: u64,
    sender_share_bps: u32,
)
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment. |
| `caller` | `Address` | Administrator authorizing the split. |
| `delivery_id` | `u64` | Identifier of the disputed escrow. |
| `sender_share_bps` | `u32` | Percentage allocation to the original sender, expressed in basis points. |

### Valid allocation range

The sender's allocation must not exceed:

```text
10,000 basis points = 100%
```

The contract rejects values above `10,000` using:

```rust
EscrowError::InvalidFee
```

The driver receives the remainder of the escrow amount.

### Distribution calculation

The sender's share is calculated using:

```rust
let sender_amount =
    record.amount.saturating_mul(sender_share_bps as i128) / 10000;

let driver_amount =
    record.amount.saturating_sub(sender_amount);
```

For example, if the escrow amount is `10,000` token units and the administrator chooses a sender allocation of `3,000` basis points:

```text
Escrow amount:       10,000
Sender allocation:   30%
Driver allocation:  70%

Sender receives:      3,000
Driver receives:      7,000
```

The calculation uses integer arithmetic, so fractional token units are truncated according to the token's integer representation.

### Balance verification

Before transferring funds, the contract checks the escrow contract's token balance:

```rust
let contract_balance =
    token::Client::new(&env, &record.token)
        .balance(&env.current_contract_address());

if contract_balance < record.amount {
    panic_with_error!(&env, EscrowError::InsufficientFunds);
}
```

If the contract does not hold enough tokens to cover the escrow amount, the operation fails with:

```rust
EscrowError::InsufficientFunds
```

### Settlement execution

The contract transfers the sender's share when it is greater than zero:

```rust
if sender_amount > 0 {
    token::Client::new(&env, &record.token).transfer(
        &env.current_contract_address(),
        &record.sender,
        &sender_amount,
    );
}
```

It then transfers the driver's share when it is greater than zero:

```rust
if driver_amount > 0 {
    token::Client::new(&env, &record.token).transfer(
        &env.current_contract_address(),
        &record.driver,
        &driver_amount,
    );
}
```

The contract subsequently:

1. Sets the escrow status to `EscrowStatus::Refunded`.
2. Saves the updated escrow record.
3. Publishes the dispute-resolution event.

### Split settlement behavior

| Sender share | Driver share | Result |
|---|---|---|
| 100% | 0% | Full refund to sender |
| 0% | 100% | Full payment to driver |
| 30% | 70% | Proportional distribution |
| 50% | 50% | Equal distribution |

**Important implementation detail:** The current split-resolution implementation does not deduct the platform fee. It distributes the original escrow amount between the sender and driver.

Also, the implementation sets the final escrow status to `Refunded` for every split, including a 100% driver allocation. This is the actual stored behavior, even though the status name may not intuitively describe every split outcome.

## 5. Dispute-resolution events

Both resolution functions publish an event using:

```rust
events::dispute_resolved(&env)
```

The current event payload is:

```rust
(caller.clone(), caller)
```

The same administrator address is therefore supplied twice.

The event does not currently encode:

- Whether the funds were released or refunded.
- The sender's split percentage.
- The amount paid to the sender.
- The amount paid to the driver.
- The final escrow status.

**Indexer integration note:** An off-chain event consumer should not assume that the current `dispute_resolved` payload identifies the selected resolution or settlement amounts. It can use the event as a resolution notification, then query the escrow record and relevant token transactions for additional details.

## 6. Failure conditions

| Condition | Result |
|---|---|
| Caller is not the stored administrator | `SwiftChainError::Unauthorized` |
| Escrow does not exist | `EscrowError::DeliveryNotFound` |
| Escrow is not paused | `EscrowError::InvalidState` |
| Sender share exceeds 10,000 basis points | `EscrowError::InvalidFee` |
| Split settlement has insufficient contract balance | `EscrowError::InsufficientFunds` |
| Token transfer fails | Transaction fails and Soroban transaction rollback semantics apply |

## 7. End-to-end dispute lifecycle

```text
1. Escrow is funded
        |
        v
2. Escrow status becomes Locked
        |
        v
3. Sender or recipient raises a dispute
        |
        v
4. Escrow status becomes Paused
        |
        v
5. Administrator reviews the dispute
        |
        v
6. Administrator selects a resolution
        |
        +-----------------------------+
        |              |              |
        v              v              v
   Driver payout   Sender refund   Proportional split
        |              |              |
        v              v              v
     Released       Refunded        Refunded
```

## 8. Source references

- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `contracts/escrow_contract/test.rs`
- `docs/architecture/smart-contract-architecture.md`
