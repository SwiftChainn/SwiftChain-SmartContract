
# Delivery Contract Initialization and Escrow Linkage

## Overview

The `delivery_contract` is responsible for creating delivery records and managing their lifecycle. During initialization, it stores the administrator address and the deployed escrow contract address so that delivery lifecycle operations can locate and invoke the escrow contract.

The initialization entry point is:

```rust
pub fn init(env: Env, admin: Address, escrow_contract: Address)
```

Implementation reference: `contracts/delivery_contract/lib.rs`.

## Initialization Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment used to access storage, ledger data, and events. |
| `admin` | `Address` | Administrator address stored for delivery-contract administration. |
| `escrow_contract` | `Address` | Deployed escrow contract address used for cross-contract calls. |

Both addresses are supplied by the deployment or initialization transaction. They are not hardcoded in the contract.

## Initialization Behavior

The `init` function performs the following operations:

1. Checks whether the `StorageKey::Admin` instance-storage entry already exists.
2. Rejects repeated initialization with `SwiftChainError::AlreadyInitialized`.
3. Stores the supplied administrator address under `StorageKey::Admin`.
4. Stores the escrow contract address under `DataKey::EscrowContract`.
5. Initializes the persistent `DataKey::DeliveryCounter` to `0`.
6. Publishes the `DeliveryContractInitialized` event.

The initialization guard is important because it prevents the contract's administrator and escrow reference from being silently overwritten by another initialization call.

### Storage entries

| Key | Storage type | Stored value | Purpose |
|---|---|---|---|
| `StorageKey::Admin` | Instance | `Address` | Identifies the delivery contract administrator. |
| `DataKey::EscrowContract` | Instance | `Address` | Identifies the escrow contract used by delivery operations. |
| `DataKey::DeliveryCounter` | Persistent | `u64` | Tracks the counter used to generate delivery IDs. |

The administrator and escrow address are stored in instance storage. The delivery counter is stored in persistent storage.

## Initialization Event

After storing the configuration, the contract publishes:

```rust
env.events().publish(
    (Symbol::new(&env, "DeliveryContractInitialized"),),
    (admin, escrow_contract),
);
```

The event has the topic `DeliveryContractInitialized` and carries the administrator and escrow contract addresses as its data.

Applications and indexers can use this event to identify the contract's initialization configuration.

## Escrow Contract Linkage

The delivery contract stores the escrow contract address under:

```rust
DataKey::EscrowContract
```

Delivery lifecycle functions retrieve this address from instance storage before invoking escrow operations.

For example, the delivery contract uses `env.invoke_contract` to call escrow entry points such as:

| Delivery operation | Escrow entry point |
|---|---|
| Recipient confirms delivery | `release_escrow` |
| Sender cancels delivery | `refund_escrow` |
| Sender or recipient raises a dispute | `raise_dispute` |

The delivery contract passes the relevant participant address and the delivery ID to the escrow entry point.

The delivery ID is converted from the shared `DeliveryId` type to `u64` before it is passed to the escrow contract.

### Example: retrieving the configured escrow address

```rust
let escrow_address: Address = env
    .storage()
    .instance()
    .get(&DataKey::EscrowContract)
    .unwrap_or_else(|| panic!("EscrowNotConfigured"));
```

If the escrow address has not been configured, the operation fails with `EscrowNotConfigured`.

## Cross-Contract Call Authorization

Soroban cross-contract calls execute in the context of the contract invocation tree. The contracts must still enforce authorization for the addresses that are permitted to perform an operation.

The delivery contract performs participant authorization checks before making its escrow calls:

- `confirm_delivery` requires the recipient's authorization and verifies that the supplied recipient matches the delivery record.
- `cancel_delivery` requires the sender's authorization and verifies that the supplied sender matches the delivery record.
- `raise_dispute` requires the caller's authorization and verifies that the caller is either the delivery sender or recipient.

The escrow contract independently performs authorization checks:

- `release_escrow` accepts the escrow administrator or the recorded recipient.
- `refund_escrow` accepts the escrow administrator or the recorded sender.
- `raise_dispute` accepts the recorded sender or recipient.

Each escrow operation also calls `require_auth()` for its supplied caller address.

**Important implementation detail:** The current escrow contract does not contain a dedicated delivery-contract allowlist or a check that restricts these lifecycle entry points to calls originating from the configured delivery contract. Authorization is based on the authenticated participant/admin address and the corresponding escrow record checks. Do not interpret the stored escrow address as an independent caller-authorization mechanism.

## Deployment and Configuration Considerations

For the integration to work correctly:

1. Deploy the escrow contract.
2. Initialize the escrow contract with its administrator, token address, and platform fee configuration.
3. Deploy the delivery contract.
4. Initialize the delivery contract with its administrator and the deployed escrow contract address.
5. Use the configured delivery contract and escrow contract addresses when creating and funding delivery records.

The escrow address supplied to `delivery_contract::init` must refer to the intended deployed escrow contract. An incorrect address can cause subsequent cross-contract operations to fail.

The delivery contract's initialization does not initialize the escrow contract. Each contract has its own initialization entry point and storage.

## Failure Conditions

| Condition | Result |
|---|---|
| Delivery contract has already been initialized | `SwiftChainError::AlreadyInitialized` |
| Escrow address is not available when a lifecycle operation needs it | `EscrowNotConfigured` |
| A participant fails the delivery contract's authorization or identity check | `NotAuthorized` |
| The escrow operation rejects the caller or escrow state | The cross-contract invocation fails and the delivery operation does not complete successfully |

## Related Source Files

- `contracts/delivery_contract/lib.rs`
- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `docs/architecture/smart-contract-architecture.md`

## Summary

The delivery contract initialization establishes the administrator, escrow contract reference, and initial delivery counter. The stored escrow address enables delivery lifecycle functions to invoke escrow operations. Authorization remains enforced by the participant checks in the delivery contract and the independent authorization and state checks in the escrow contract.
