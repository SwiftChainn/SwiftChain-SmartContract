
# Escrow Integration Test Reference

## Overview

This document describes the escrow contract test suite, its authorization setup, and the scenarios covered by the lifecycle tests.

The tests are implemented in:

`contracts/escrow_contract/test.rs`

The escrow contract tests use the Soroban SDK testing environment and a Stellar Asset Contract to simulate token transfers.

## 1. Test Environment Setup

The test suite uses a shared setup helper:

```rust
fn setup_env() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EscrowContract, ());

    (env, contract_id)
}
```

The helper performs three operations:

1. Creates a default Soroban test environment.
2. Enables mocked authorization.
3. Registers the escrow contract and returns its address.

### 1.1 Understanding mock_all_auths()

The Soroban SDK method:

```rust
env.mock_all_auths();
```

enables authorization mocking within the test environment.

This allows contract calls that require address authorization to execute without requiring a real wallet signature for every test operation.

For example, the escrow contract requires authorization for operations such as:

- Initializing the protocol administrator.
- Updating platform fees.
- Creating an escrow.
- Releasing an escrow.
- Refunding an escrow.
- Raising a dispute.
- Resolving a dispute.

Mocked authorization makes it easier to test contract behavior and state transitions.

**Important:** `mock_all_auths()` is a testing utility. It does not remove authorization requirements from deployed contracts and must not be treated as a production security mechanism.

Tests that need to verify unauthorized access should explicitly exercise the contract's authorization checks and assert the expected failure.

## 2. Test Helper Functions

The escrow test suite provides the following helpers.

### 2.1 setup_env()

Creates a test environment, enables mocked authorization, and registers the escrow contract.

### 2.2 setup_token()

```rust
fn setup_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}
```

Registers a Stellar Asset Contract for testing token transfers.

### 2.3 mint()

```rust
fn mint(
    env: &Env,
    token: &Address,
    to: &Address,
    amount: i128,
)
```

Mints test tokens to a specified address.

### 2.4 balance()

```rust
fn balance(
    env: &Env,
    token: &Address,
    of: &Address,
) -> i128
```

Reads a token balance using the Soroban token client.

These helpers allow the tests to verify escrow deposits, payouts, refunds, and platform fee transfers.

## 3. Escrow Test Inventory

The current `contracts/escrow_contract/test.rs` contains the following 14 test functions.

### 3.1 Initialization and Configuration

**Test 1: `test_init_and_platform_fee_default`**

Verifies that:

- The escrow contract initializes successfully.
- The administrator is stored.
- The default platform fee is zero.

**Test 2: `test_update_platform_fee_success`**

Verifies that the administrator can update the platform fee to a valid value.

**Test 3: `test_update_platform_fee_invalid_value`**

Verifies that an invalid platform fee exceeding the maximum allowed value is rejected with `EscrowError::InvalidFee`.

### 3.2 Escrow Creation and Locking

**Test 4: `test_create_escrow_locks_funds_and_persists_record`**

Verifies that:

- The sender's tokens are transferred into escrow.
- The contract receives the deposited amount.
- The escrow record is persisted.
- The record contains the expected parties, amount, status, and timestamps.

**Test 5: `test_create_escrow_duplicate_delivery_rejected`**

Verifies that creating another escrow with an existing delivery identifier is rejected with `EscrowError::DuplicateDelivery`.

### 3.3 Escrow Release

**Test 6: `test_release_escrow_by_recipient_with_platform_fee_split`**

Verifies that:

- The recipient can authorize escrow release.
- The driver receives the net payout.
- The administrator receives the platform fee.
- The escrow status changes to `Released`.

**Test 7: `test_release_escrow_unauthorized_rejected`**

Verifies that an unauthorized caller cannot release an escrow.

The expected error is `SwiftChainError::Unauthorized`.

**Test 8: `test_release_from_paused_state_rejected_with_invalid_state`**

Verifies that an escrow in the `Paused` state cannot be released through the normal release operation.

The expected error is `EscrowError::InvalidState`.

### 3.4 Refund and Dispute Handling

**Test 9: `test_refund_escrow_by_sender_full_amount_no_fee`**

Verifies that:

- The sender can refund a locked escrow.
- The full escrow amount is returned.
- No platform fee is deducted.
- The escrow status changes to `Refunded`.

**Test 10: `test_raise_dispute_pauses_escrow_and_records_metadata`**

Verifies that raising a dispute:

- Changes the escrow status to `Paused`.
- Stores the address that raised the dispute.
- Records the dispute timestamp.

**Test 11: `test_refund_from_paused_state_by_admin_allowed`**

Verifies that the administrator can refund an escrow from the `Paused` state.

**Test 12: `test_refund_on_released_escrow_rejected_with_invalid_state`**

Verifies that a previously released escrow cannot be refunded.

The expected error is `EscrowError::InvalidState`.

### 3.5 Balance Protection and Queries

**Test 13: `test_insufficient_funds_guard_on_release`**

Verifies that escrow release is rejected when the stored escrow amount exceeds the contract's available token balance.

The expected error is `EscrowError::InsufficientFunds`.

**Test 14: `test_get_escrow_not_found`**

Verifies that querying a nonexistent delivery identifier returns `EscrowError::DeliveryNotFound`.

## 4. Lifecycle Coverage Matrix

| Lifecycle operation | Test coverage | Expected behavior |
|---|---|---|
| Initialization | Tests 1–3 | Initializes protocol configuration and validates fee updates. |
| Lock | Tests 4–5 | Locks funds, persists escrow state, and rejects duplicate identifiers. |
| Release | Tests 6–8, 13 | Pays the driver, distributes fees, and rejects unauthorized, invalid-state, or insufficient-balance operations. |
| Refund | Tests 9, 11–12 | Returns funds to the sender and rejects invalid refund states. |
| Split | No dedicated escrow split test in the current test file | Requires separate verification of split-dispute resolution. |
| Dispute | Test 10 | Pauses escrow and records dispute metadata. |
| Query | Test 14 | Rejects queries for nonexistent escrow records. |

### Important Coverage Limitation

The current escrow test file does not contain a dedicated test for `resolve_dispute_split`.

It also does not contain a dedicated test for the settlement-contract integration.

These scenarios should not be presented as covered by the existing 14 tests.

The settlement and split functionality can be covered by additional integration tests in future test-suite updates.

## 5. Test Coverage Summary

| Category | Number of tests |
|---|---:|
| Initialization and configuration | 3 |
| Escrow creation and locking | 2 |
| Escrow release | 3 |
| Refund and dispute handling | 4 |
| Balance protection and query | 2 |
| **Total currently present** | **14** |

The issue description refers to 16 integration tests. However, the current escrow test source contains 14 functions annotated with `#[test]`.

Two additional tests would need to be identified or implemented before the repository can accurately claim 16 escrow tests.

This documentation deliberately reflects the source code rather than reporting tests that do not exist.

## 6. Running the Tests

From the repository root, run:

```bash
cargo test -p escrow_contract
```

To display test output:

```bash
cargo test -p escrow_contract -- --nocapture
```

To run a specific test:

```bash
cargo test -p escrow_contract test_create_escrow_locks_funds_and_persists_record
```

The repository's `TESTING.md` also documents the broader test workflow and known Soroban dependency compatibility limitations.

## 7. Related Contract Functions

The test suite validates behavior implemented by the following escrow functions:

| Function | Purpose |
|---|---|
| `init` | Initializes the escrow contract. |
| `update_platform_fee` | Updates the platform fee. |
| `get_platform_fee` | Retrieves the configured platform fee. |
| `create_escrow` | Creates an escrow and locks tokens. |
| `release_escrow` | Processes driver payment. |
| `refund_escrow` | Returns funds to the sender. |
| `raise_dispute` | Pauses escrow for dispute handling. |
| `resolve_dispute` | Resolves a dispute through release or refund. |
| `resolve_dispute_split` | Distributes funds between the sender and driver. |
| `get_escrow` | Retrieves an escrow record. |

## 8. Source References

- `contracts/escrow_contract/lib.rs`
- `contracts/escrow_contract/test.rs`
- `contracts/shared_types/lib.rs`
- `TESTING.md`
- `docs/architecture/smart-contract-architecture.md`
