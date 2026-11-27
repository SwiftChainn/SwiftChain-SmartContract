
# Delivery Contract Integration Test Reference

## Overview

This document describes the delivery contract integration tests and the lifecycle scenarios covered by the current test suite.

The tests are located in:

```text
contracts/delivery_contract/test.rs
```

The delivery contract implementation is located in:

```text
contracts/delivery_contract/lib.rs
```

The test suite uses Soroban's test environment and mock contracts to exercise delivery operations and cross-contract interactions.

## Test Suite Count

The current source file contains **16 integration tests**.

The issue description refers to 13 tests, but the repository currently defines 16 functions annotated with `#[test]`. This document reflects the current source code rather than limiting the reference to the older count.

## 1. Test Setup and Mock Contracts

### Test Environment

The test suite uses:

```rust
let env = Env::default();
```

The shared setup helper:

```rust
fn setup_full(...)
```

performs the following operations:

1. Enables mocked authorization using `env.mock_all_auths()`.
2. Registers a mock escrow contract.
3. Registers a mock reputation contract.
4. Registers the delivery contract.
5. Creates test addresses for the shipper, driver, and recipient.
6. Initializes the delivery contract with the shipper as admin and the mock escrow contract address.

### Mock Escrow Contract

The mock escrow contract provides methods used by the delivery contract:

- `refund_escrow`
- `release_escrow`
- `raise_dispute`

The mock records calls in temporary storage so tests can verify that the expected escrow operation was invoked.

### Mock Reputation Contract

The mock reputation contract provides:

- `increase_reputation`
- `decrease_reputation`

These methods record the driver address in temporary storage for test assertions.

The mock does not represent a full deployment of the identity/reputation contract.

## 2. Complete Integration Test Inventory

The following table lists all 16 tests currently defined in `contracts/delivery_contract/test.rs`.

| # | Test function | Scenario | Expected result |
|---|---|---|---|
| 1 | `test_happy_path_full_lifecycle` | Create, assign, transit, confirm, and release escrow. | Delivery becomes `Delivered`; escrow release is recorded. |
| 2 | `test_cancellation_after_assign` | Cancel a delivery after driver assignment. | Delivery becomes `Cancelled`; escrow refund is recorded. |
| 3 | `test_dispute_path` | Raise a dispute after assignment and transit. | Delivery becomes `Disputed`; escrow dispute is recorded. |
| 4 | `test_invalid_assign_when_delivered` | Attempt to assign a driver after delivery completion. | Operation fails with `InvalidState`. |
| 5 | `test_invalid_mark_in_transit_without_assign` | Attempt to start transit without an assigned driver. | Operation fails with `NotAuthorized`. |
| 6 | `test_invalid_confirm_without_transit` | Attempt to confirm delivery before transit. | Operation fails with `InvalidState`. |
| 7 | `test_invalid_dispute_when_cancelled` | Attempt to dispute a cancelled delivery. | Operation fails with `InvalidState`. |
| 8 | `test_invalid_cancel_when_delivered` | Attempt to cancel a delivered delivery. | Operation fails with `InvalidState`. |
| 9 | `test_unauthorized_assign_driver` | An unauthorized address attempts driver assignment. | Operation fails with `NotAuthorized`. |
| 10 | `test_unauthorized_mark_in_transit` | An address other than the assigned driver attempts to start transit. | Operation fails with `NotAuthorized`. |
| 11 | `test_unauthorized_confirm_delivery` | An address other than the recipient attempts confirmation. | Operation fails with `NotAuthorized`. |
| 12 | `test_unauthorized_raise_dispute` | An unrelated address attempts to raise a dispute. | Operation fails with `NotAuthorized`. |
| 13 | `test_unauthorized_cancel_delivery` | An address other than the sender attempts cancellation. | Operation fails with `NotAuthorized`. |
| 14 | `test_dispute_then_resolve_increments_reputation` | Exercise the dispute path and the mock reputation-increase assertion. | The test verifies the expected driver address is recorded for the increase scenario. |
| 15 | `test_dispute_then_resolve_penalizes_driver` | Exercise the dispute path and the mock reputation-decrease assertion. | The test verifies the expected driver address is recorded for the decrease scenario. |
| 16 | `test_create_delivery_missing_fields` | Create a delivery with empty origin/destination strings and zero cargo weight. | Delivery creation succeeds and returns the generated delivery identifier. |

## 3. Delivery Lifecycle

The delivery contract supports the following primary lifecycle:

```text
Create Delivery
      |
      v
   Pending
      |
      v
 Assign Driver
      |
      v
    Active
      |
      v
  Mark In Transit
      |
      v
  InTransit
      |
      v
Confirm Delivery
      |
      v
  Delivered
      |
      v
 Release Escrow
```

The contract also supports cancellation and dispute paths.

### Cancellation Path

```text
Pending or Active
       |
       v
Cancel Delivery
       |
       v
   Cancelled
       |
       v
 Refund Escrow
```

The actual allowed transition depends on the delivery's current state. A delivered or cancelled delivery is terminal.

### Dispute Path

```text
Active or InTransit
        |
        v
  Raise Dispute
        |
        v
    Disputed
        |
        v
Dispute Resolution
```

The delivery contract invokes the configured escrow contract when a dispute is raised.

## 4. Lifecycle Coverage Matrix

| Lifecycle stage | Related test(s) | Coverage |
|---|---|---|
| Delivery creation | `test_happy_path_full_lifecycle`, `test_cancellation_after_assign`, `test_dispute_path`, `test_create_delivery_missing_fields` | Successful creation and acceptance of empty/zero metadata values. |
| Driver assignment | `test_happy_path_full_lifecycle`, `test_invalid_assign_when_delivered`, `test_unauthorized_assign_driver` | Successful assignment, invalid-state rejection, and authorization rejection. |
| Transit | `test_happy_path_full_lifecycle`, `test_invalid_mark_in_transit_without_assign`, `test_unauthorized_mark_in_transit` | Successful transit start, missing/incorrect driver rejection, and authorization enforcement. |
| Delivery confirmation | `test_happy_path_full_lifecycle`, `test_invalid_confirm_without_transit`, `test_unauthorized_confirm_delivery` | Successful confirmation, invalid-state rejection, and recipient authorization. |
| Escrow release | `test_happy_path_full_lifecycle` | Verifies the mock escrow release call after confirmation. |
| Cancellation | `test_cancellation_after_assign`, `test_invalid_cancel_when_delivered`, `test_unauthorized_cancel_delivery` | Successful cancellation, invalid-state rejection, and sender authorization. |
| Escrow refund | `test_cancellation_after_assign` | Verifies the mock escrow refund call after cancellation. |
| Dispute | `test_dispute_path`, `test_invalid_dispute_when_cancelled`, `test_unauthorized_raise_dispute` | Successful dispute, invalid-state rejection, and caller authorization. |
| Reputation integration scenarios | `test_dispute_then_resolve_increments_reputation`, `test_dispute_then_resolve_penalizes_driver` | Exercises mock reputation recording for increase and decrease scenarios. |

## 5. Test Details by Lifecycle Stage

### A. Creation and Happy Path

`test_happy_path_full_lifecycle` verifies the complete successful delivery path:

1. Creates a delivery.
2. Assigns a driver.
3. Marks the delivery as in transit.
4. Confirms delivery through the recipient.
5. Queries the delivery record.
6. Verifies that its status is `Delivered`.
7. Verifies that the mock escrow contract recorded a release.

`test_create_delivery_missing_fields` checks that the current implementation accepts empty origin and destination strings and zero cargo weight. The test does not establish that these values are valid business data; it documents the current contract behavior.

### B. Cancellation

`test_cancellation_after_assign` verifies that the sender can cancel an assigned delivery and that the mock escrow contract records a refund.

`test_invalid_cancel_when_delivered` verifies that a delivered record cannot be cancelled.

`test_unauthorized_cancel_delivery` verifies that an unrelated address cannot cancel a delivery on behalf of the sender.

### C. Disputes

`test_dispute_path` verifies that a dispute can be raised after driver assignment and transit have started. It also checks that the mock escrow contract records the dispute operation.

`test_invalid_dispute_when_cancelled` verifies that a cancelled delivery cannot transition into a disputed state.

`test_unauthorized_raise_dispute` verifies that an unrelated address cannot raise a dispute for the delivery.

### D. Authorization and State Validation

The tests verify that the contract rejects operations when the caller is not permitted or when the requested state transition is invalid.

The covered authorization failures include:

- Unauthorized driver assignment.
- Unauthorized transit updates.
- Unauthorized delivery confirmation.
- Unauthorized dispute creation.
- Unauthorized cancellation.

The covered invalid transitions include:

- Assigning a driver after delivery.
- Starting transit without a valid assigned driver.
- Confirming before transit.
- Raising a dispute after cancellation.
- Cancelling after delivery.

### E. Reputation Integration Scenarios

The two reputation-related tests exercise the mock reputation contract's recording behavior.

They verify the driver address associated with the increase and decrease scenarios.

Important limitation: these tests do not execute a complete dispute-resolution contract workflow or assert a changed reputation score in the real identity/reputation contract. They should therefore be described as mock integration scenarios, not end-to-end verification of the actual reputation contract's scoring logic.

## 6. State Transition Reference

The delivery contract's `validate_transition` function currently permits:

| Current state | Allowed next state(s) |
|---|---|
| `Pending` | `Active`, `Cancelled` |
| `Active` | `InTransit`, `Disputed`, `Cancelled` |
| `InTransit` | `Delivered`, `Disputed` |
| `Disputed` | `Delivered`, `Cancelled` |
| `Delivered` | None |
| `Cancelled` | None |

Any transition not listed above is rejected with `InvalidState`.

## 7. Running the Tests

Run the delivery contract tests using the repository's existing Rust workspace and test configuration.

From the repository root, the standard Cargo test command is:

```bash
cargo test -p delivery_contract
```

If the package name differs in the workspace manifest, use the package name declared in the relevant `Cargo.toml`.

The test suite should be run with the repository's configured Rust toolchain and Soroban dependencies.

## 8. Source References

- `contracts/delivery_contract/lib.rs`
- `contracts/delivery_contract/test.rs`
- `contracts/shared_types/lib.rs`
- `contracts/identity_reputation_contract/lib.rs`
- `docs/architecture/smart-contract-architecture.md`
