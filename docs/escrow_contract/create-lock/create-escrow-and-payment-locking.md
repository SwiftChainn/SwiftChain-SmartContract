
# Escrow Creation and Payment Locking

## Overview

The escrow creation flow locks a sender's tokens in the Escrow Contract and creates a persistent escrow record associated with a delivery identifier.

The implementation is provided by `create_escrow` in:

- `contracts/escrow_contract/lib.rs`
- Shared escrow types and storage helpers: `contracts/shared_types/lib.rs`

Escrow creation is a separate operation from delivery creation. The delivery contract creates the delivery record, while the escrow contract independently receives and locks the payment.

## 1. Contract Function Signature

The escrow contract exposes the following function:

```rust
pub fn create_escrow(
    env: Env,
    sender: Address,
    recipient: Address,
    driver: Address,
    delivery_id: u64,
    token: Address,
    amount: i128,
)
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment |
| `sender` | `Address` | Account funding the escrow |
| `recipient` | `Address` | Intended recipient of the delivery |
| `driver` | `Address` | Driver associated with the escrow |
| `delivery_id` | `u64` | Unique delivery identifier |
| `token` | `Address` | Stellar token contract address |
| `amount` | `i128` | Amount to transfer into escrow |

The `delivery_id` connects the escrow record to the corresponding delivery record.

## 2. Authorization Requirements

The sender must authorize the transaction.

The contract enforces this requirement through:

```rust
sender.require_auth();
```

This ensures that the sender explicitly authorizes the operation that transfers their tokens into escrow.

The caller cannot fund an escrow using another account's tokens without the required authorization.

## 3. Duplicate Delivery Protection

Before transferring tokens, the contract checks whether an escrow record already exists for the supplied delivery identifier.

```rust
if env.storage().persistent().has(&escrow_key(delivery_id)) {
    panic_with_error!(&env, EscrowError::DuplicateDelivery);
}
```

If the identifier already exists, the transaction fails with `EscrowError::DuplicateDelivery`.

This prevents multiple escrow records from being created for the same delivery identifier.

## 4. Token Transfer and Payment Locking

After authorization and duplicate-record validation, the contract transfers tokens from the sender to its own contract address.

```rust
token::Client::new(&env, &token).transfer(
    &sender,
    &env.current_contract_address(),
    &amount,
);
```

The transfer uses the Stellar token contract client.

The transfer has two important effects:

1. Tokens are debited from the sender.
2. Tokens are credited to the escrow contract.

The escrow contract becomes the holder of the deposited funds until an authorized release, refund, or dispute-resolution operation occurs.

The token transfer requires the appropriate sender authorization for the token operation.

**Important:** The current implementation does not explicitly validate that `amount` is positive or verify that the supplied token matches the token stored in `ProtocolConfig`. Integrators should use the configured asset and a valid positive amount. These checks should not be assumed to exist in the current implementation.

## 5. Persistent Escrow Record

After the token transfer succeeds, the contract stores an `EscrowRecord`.

```rust
save_escrow(
    &env,
    delivery_id,
    &EscrowRecord {
        sender: sender.clone(),
        recipient: recipient.clone(),
        driver,
        token,
        amount,
        status: EscrowStatus::Locked,
        created_at: env.ledger().timestamp(),
        disputed_by: None,
        disputed_at: None,
    },
);
```

### EscrowRecord fields

| Field | Description |
|---|---|
| `sender` | Original account funding the escrow |
| `recipient` | Intended delivery recipient |
| `driver` | Driver associated with the delivery |
| `token` | Token contract used for payment |
| `amount` | Amount deposited into escrow |
| `status` | Initial escrow state: `Locked` |
| `created_at` | Ledger timestamp when the record was created |
| `disputed_by` | Initially `None` |
| `disputed_at` | Initially `None` |

### Storage implementation

The `save_escrow` helper uses the persistent storage API:

```rust
let key = escrow_key(delivery_id);
env.storage().persistent().set(&key, record);
```

It also extends the record's time-to-live (TTL):

```rust
env.storage().persistent().extend_ttl(
    &key,
    constants::ESCROW_TTL_THRESHOLD,
    constants::ESCROW_TTL_EXTEND_TO,
);
```

The current threshold and extension values are both `518400` ledgers.

The storage key is generated through `escrow_key(delivery_id)` in `shared_types`.

## 6. Escrow Funding Event

After the record is stored, the contract publishes an escrow funding event.

```rust
env.events().publish(
    (events::escrow_funded(&env), delivery_id),
    (sender, recipient, amount),
);
```

### Event structure

| Component | Value |
|---|---|
| Topic 1 | `escrow_funded` |
| Topic 2 | `delivery_id` |
| Data 1 | `sender` |
| Data 2 | `recipient` |
| Data 3 | `amount` |

This event allows external applications and indexers to identify newly funded escrows.

The event topic helper is defined in:

`contracts/shared_types/lib.rs`

## 7. Interplay with Delivery Creation

Delivery creation and escrow creation are separate operations.

The delivery contract exposes:

```rust
pub fn create_delivery(
    env: Env,
    sender: Address,
    recipient: Address,
    metadata: DeliveryMetadata,
) -> DeliveryId
```

This operation:

1. Requires sender authorization.
2. Generates a delivery identifier.
3. Creates a delivery record.
4. Stores the delivery record in persistent storage.
5. Emits the `delivery_created` event.
6. Returns the generated `DeliveryId`.

It does not automatically call `create_escrow`.

### Recommended application-level sequence

```text
Sender
  |
  v
Delivery Contract
  |
  | create_delivery(...)
  v
Delivery Record Created
  |
  | Returns DeliveryId
  v
Escrow Contract
  |
  | create_escrow(..., delivery_id, token, amount)
  v
Sender Authorization
  |
  v
Token Transfer
  |
  v
Escrow Record Stored
  |
  v
escrow_funded Event
```

The application or transaction orchestration layer must coordinate both operations and use the returned delivery identifier when creating the escrow.

Because these are separate contract invocations, the application must handle failures and partial completion explicitly. It must not assume that creating a delivery automatically locks its payment.

## 8. Retrieving an Escrow Record

The contract exposes:

```rust
pub fn get_escrow(env: Env, delivery_id: u64) -> EscrowRecord
```

This retrieves the persistent escrow record associated with the supplied delivery identifier.

If no record exists, the contract raises:

`EscrowError::DeliveryNotFound`

The retrieval helper also extends the record's TTL.

## 9. Related Lifecycle Operations

Once the escrow is created, the following functions manage its lifecycle:

| Function | Purpose |
|---|---|
| `release_escrow` | Releases payment to the driver and transfers any platform fee to the admin |
| `refund_escrow` | Returns escrow funds to the original sender |
| `raise_dispute` | Pauses a locked escrow and records dispute information |
| `resolve_dispute` | Resolves a paused escrow through an admin decision |
| `resolve_dispute_split` | Distributes escrow funds between sender and driver |
| `get_escrow` | Retrieves the escrow record |

See the escrow event documentation for the corresponding emitted events.

## 10. Security Considerations

- Sender authorization is required before escrow creation.
- Duplicate delivery identifiers are rejected.
- Funds are transferred to the escrow contract rather than directly to the driver.
- Escrow records are persisted using Soroban persistent storage.
- Escrow operations validate the current escrow state before performing their corresponding transitions.
- Applications should verify transaction success before reporting that payment has been locked.
- Applications should use the intended token contract and a valid positive amount.

## Source References

- `contracts/escrow_contract/lib.rs`
- `contracts/escrow_contract/test.rs`
- `contracts/delivery_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `docs/architecture/smart-contract-architecture.md`
