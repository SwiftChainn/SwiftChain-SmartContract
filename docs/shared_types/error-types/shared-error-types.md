
# SwiftChain Shared and Contract-Specific Error Types

## 1. Overview

SwiftChain uses Soroban contract errors to communicate failed operations and invalid protocol states.

The repository defines:

- A shared `SwiftChainError` enum in `contracts/shared_types/lib.rs`.
- Contract-specific error enums in the escrow, delivery, fleet-management, and settlement contracts.
- The Soroban `panic_with_error!` macro to terminate an operation with a typed contract error.

The error enums use `#[contracterror]` and `#[repr(u32)]`, allowing error variants to be represented by numeric contract error codes.

Not every contract currently defines a separate error enum. The dispute-resolution and identity/reputation contracts primarily use `SwiftChainError` in the inspected implementation.

## 2. Shared SwiftChainError

**Source:** `contracts/shared_types/lib.rs`

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SwiftChainError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    DeliveryNotFound = 4,
    InvalidState = 5,
    InsufficientFunds = 6,
    EscrowLocked = 7,
    DuplicateDelivery = 8,
    ProviderNotFound = 9,
    InvalidAddress = 10,
}
```

| Numeric code | Variant | Meaning |
|---:|---|---|
| 1 | `Unauthorized` | The caller is not authorized to perform the requested operation. |
| 2 | `AlreadyInitialized` | The contract or protocol state has already been initialized. |
| 3 | `NotInitialized` | Required contract or protocol state has not been initialized. |
| 4 | `DeliveryNotFound` | The requested delivery or related record could not be found. |
| 5 | `InvalidState` | The requested operation is invalid for the current state. |
| 6 | `InsufficientFunds` | The contract balance is insufficient for the requested transfer. |
| 7 | `EscrowLocked` | Escrow funds are locked and cannot be released or refunded in the requested context. |
| 8 | `DuplicateDelivery` | The delivery identifier already exists. |
| 9 | `ProviderNotFound` | The requested provider or driver record could not be found. |
| 10 | `InvalidAddress` | An address argument is invalid for the requested operation. |

The shared error enum allows multiple contracts to use consistent error meanings for common protocol-level failures.

## 3. Contract-specific errors

### 3.1 EscrowError

**Source:** `contracts/escrow_contract/lib.rs`

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    InvalidState = 1,
    DeliveryNotFound = 2,
    InsufficientFunds = 3,
    DuplicateDelivery = 4,
    InvalidFee = 5,
}
```

| Numeric code | Variant | Meaning |
|---:|---|---|
| 1 | `InvalidState` | The escrow is not in a state that permits the requested operation. |
| 2 | `DeliveryNotFound` | The escrow record for the requested delivery identifier does not exist. |
| 3 | `InsufficientFunds` | The escrow contract's token balance is insufficient for the operation. |
| 4 | `DuplicateDelivery` | An escrow record already exists for the delivery identifier. |
| 5 | `InvalidFee` | A supplied platform fee value is invalid. |

The escrow contract uses `EscrowError` for escrow-specific validation and uses `SwiftChainError` for shared protocol failures such as authorization and initialization errors.

### 3.2 DeliveryError

**Source:** `contracts/delivery_contract/lib.rs`

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DeliveryError {
    InvalidState = 1,
}
```

| Numeric code | Variant | Meaning |
|---:|---|---|
| 1 | `InvalidState` | The requested delivery lifecycle transition is not permitted. |

The delivery contract also uses shared errors such as `Unauthorized`, `AlreadyInitialized`, and `DeliveryNotFound`.

### 3.3 FleetError

**Source:** `contracts/fleet_management_contract/lib.rs`

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FleetError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    FleetNotFound = 4,
    DriverAlreadyInvited = 5,
    InviteNotFound = 6,
    DriverAlreadyActive = 7,
}
```

| Numeric code | Variant | Meaning |
|---:|---|---|
| 1 | `AlreadyInitialized` | The fleet-management contract has already been initialized. |
| 2 | `NotInitialized` | Required fleet-management configuration has not been initialized. |
| 3 | `Unauthorized` | The caller is not authorized to perform the requested fleet operation. |
| 4 | `FleetNotFound` | The requested fleet record does not exist. |
| 5 | `DriverAlreadyInvited` | The driver already has a pending invitation for the fleet. |
| 6 | `InviteNotFound` | The expected driver invitation does not exist. |
| 7 | `DriverAlreadyActive` | The driver is already an active member of the fleet. |

### 3.4 SettlementError

**Source:** `contracts/settlement_contract/lib.rs`

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SettlementError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    UnsupportedAssetPair = 4,
    InvalidAmount = 5,
    SlippageExceeded = 6,
}
```

| Numeric code | Variant | Meaning |
|---:|---|---|
| 1 | `AlreadyInitialized` | The settlement contract has already been initialized. |
| 2 | `NotInitialized` | Required settlement configuration has not been initialized. |
| 3 | `Unauthorized` | The caller is not authorized to perform the requested settlement operation. |
| 4 | `UnsupportedAssetPair` | The requested source and destination asset pair is not supported. |
| 5 | `InvalidAmount` | An amount, fee, or related numeric argument is invalid. |
| 6 | `SlippageExceeded` | The swap output does not satisfy the requested minimum amount. |

## 4. Contracts using shared errors without a local contracterror enum

### Dispute resolution

**Source:** `contracts/dispute_resolution_contract/lib.rs`

The inspected dispute-resolution implementation uses `SwiftChainError` for failures such as:

- `AlreadyInitialized`
- `Unauthorized`
- `NotInitialized`
- Other shared protocol validation failures where applicable

The current module does not define a separate `#[contracterror]` enum.

### Identity and reputation

**Source:** `contracts/identity_reputation_contract/lib.rs`

The inspected identity/reputation implementation also uses `SwiftChainError` for shared failures such as:

- `AlreadyInitialized`
- `Unauthorized`
- `NotInitialized`
- `ProviderNotFound`

The current module does not define a separate `#[contracterror]` enum.

## 5. Why explicit u32 discriminants matter

Each error variant is assigned an explicit integer:

```rust
#[repr(u32)]
pub enum ExampleError {
    Unauthorized = 1,
    InvalidState = 2,
}
```

The numeric discriminant is the stable code associated with the error variant.

Explicit values make error handling more predictable because a caller can identify an error using its numeric code instead of relying on Rust source ordering or a debug-formatted string.

For example:

```rust
panic_with_error!(env, SwiftChainError::Unauthorized);
```

causes the contract invocation to fail with the corresponding contract error.

The numeric value is not a globally unique identifier across the entire SwiftChain protocol.

For example:

- `SwiftChainError::Unauthorized` has code `1`.
- `EscrowError::InvalidState` also has code `1`.
- `SettlementError::AlreadyInitialized` also has code `1`.

These codes belong to different error enums. A client should interpret an error using the contract that produced it and the relevant contract error definition.

Do not decode an error using only its numeric value without considering the originating contract.

## 6. panic_with_error! convention

The repository imports and uses Soroban's `panic_with_error!` macro.

Example:

```rust
if caller != stored_admin {
    panic_with_error!(env, SwiftChainError::Unauthorized);
}
```

The macro terminates the current contract invocation with the specified typed contract error.

It is used for validation failures such as:

- Unauthorized callers.
- Missing configuration.
- Missing records.
- Invalid state transitions.
- Insufficient balances.
- Unsupported asset pairs.
- Invalid fee values.

The contract should use the most relevant error enum:

- Use `SwiftChainError` for shared protocol-level errors.
- Use the local contract error enum for contract-specific failures.

The macro should not be replaced with arbitrary string panics when a defined typed error already represents the failure.

Some source paths also contain ordinary `panic!` or `expect` calls. This document describes the typed error convention and does not claim every failure path in the repository currently uses `panic_with_error!`.

## 7. How errors surface to backend and frontend applications

A Soroban contract invocation can return a failed result containing a contract error.

The backend or frontend calling the contract should:

1. Detect that the invocation failed.
2. Identify the contract that produced the failure.
3. Read the contract error code from the returned Soroban error information.
4. Decode the code using the matching contract error enum.
5. Convert the technical error into an appropriate user-facing message.
6. Avoid treating a numeric code from one contract as if it necessarily has the same meaning in another contract.

A conceptual mapping could look like this:

| Contract | Error enum | Example code | Suggested application handling |
|---|---|---:|---|
| Shared protocol | `SwiftChainError` | 1 | Explain that the caller is not authorized. |
| Escrow | `EscrowError` | 3 | Explain that escrow funds are insufficient. |
| Delivery | `DeliveryError` | 1 | Explain that the delivery transition is invalid. |
| Fleet management | `FleetError` | 4 | Explain that the fleet could not be found. |
| Settlement | `SettlementError` | 6 | Explain that the swap exceeded the permitted slippage. |

These are illustrative application mappings, not an existing backend/frontend implementation.

The inspected smart-contract repository does not contain a verified backend or frontend error-decoding layer. The consuming application must implement and test that mapping against actual Soroban invocation responses.

## 8. Error compatibility guidelines

When adding or changing contract errors:

1. Preserve existing numeric discriminants.
2. Do not reuse an existing discriminant for a different meaning.
3. Add new variants with unused numeric values.
4. Keep shared errors in `SwiftChainError` when they represent common protocol failures.
5. Keep contract-specific validation errors in the relevant contract module.
6. Update this document whenever an error variant or discriminant changes.
7. Ensure backend and frontend decoders are updated when a new error is introduced.
8. Avoid relying on human-readable panic text as the primary error identifier.

Changing an existing error discriminant can break clients that decode contract failures by numeric code.

## 9. Source references

- `contracts/shared_types/lib.rs`
- `contracts/escrow_contract/lib.rs`
- `contracts/delivery_contract/lib.rs`
- `contracts/fleet_management_contract/lib.rs`
- `contracts/settlement_contract/lib.rs`
- `contracts/dispute_resolution_contract/lib.rs`
- `contracts/identity_reputation_contract/lib.rs`
- `docs/architecture/smart-contract-architecture.md`
