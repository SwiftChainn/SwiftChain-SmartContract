
# Driver Assignment to Delivery Order

## Overview

The delivery contract provides `assign_driver` to associate a driver address with an existing delivery order.

The function supports administrator assignment and driver self-assignment.

A successful assignment updates the delivery record, changes its status to `Active`, and emits the `driver_assigned` event.

**Implementation reference:** `contracts/delivery_contract/lib.rs`

**Data structure reference:** `contracts/shared_types/lib.rs`

## 1. Function Signature

```rust
pub fn assign_driver(
    env: Env,
    caller: Address,
    delivery_id: DeliveryId,
    driver: Address,
)
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment. |
| `caller` | `Address` | Address authorizing the assignment. |
| `delivery_id` | `DeliveryId` | Identifier of the delivery order. |
| `driver` | `Address` | Address of the driver being assigned. |

`DeliveryId` is a shared type defined in `contracts/shared_types/lib.rs` and wraps a `u64` identifier.

## 2. Authorization Requirements

The function first requires authorization from the supplied caller:

```rust
caller.require_auth();
```

It then determines whether the caller is an administrator or is assigning themselves as the driver.

```rust
let is_admin = Self::is_admin(&env, &caller);
let is_self_assignment = caller == driver;

if !is_admin && !is_self_assignment {
    panic!("NotAuthorized");
}
```

### Supported Assignment Scenarios

**Administrator assignment**

The registered administrator can assign another address as the delivery driver.

**Driver self-assignment**

A driver can assign themselves by supplying their own address as both `caller` and `driver`.

**Unauthorized assignment**

A caller who is neither the administrator nor the driver being assigned is rejected.

The function uses the contract's stored administrator address through `Self::is_admin`.

## 3. Delivery State Validation

After authorization, the contract retrieves the delivery record from persistent storage.

```rust
let key = delivery_key(delivery_id);

let mut delivery: DeliveryRecord = env
    .storage()
    .persistent()
    .get(&key)
    .unwrap_or_else(|| panic!("DeliveryNotFound"));
```

If the delivery does not exist, the operation fails with `DeliveryNotFound`.

The contract then validates the requested transition:

```rust
validate_transition(
    delivery.status.clone(),
    DeliveryStatus::Active,
)
.unwrap_or_else(|_| panic!("InvalidState"));
```

The delivery state machine permits the following transitions:

| Current status | Target status | Assignment allowed |
|---|---|---|
| `Pending` | `Active` | Yes |
| `Active` | `Active` | No |
| `InTransit` | `Active` | No |
| `Delivered` | `Active` | No |
| `Disputed` | `Active` | No |
| `Cancelled` | `Active` | No |

The transition is validated using `validate_transition` in `contracts/delivery_contract/lib.rs`.

A delivery must therefore be in the `Pending` state before a driver can be assigned.

## 4. Updating the Delivery Record

Once authorization and state validation succeed, the contract updates the delivery:

```rust
delivery.driver = Some(driver.clone());
delivery.status = DeliveryStatus::Active;
```

The updated record is saved to persistent storage:

```rust
env.storage()
    .persistent()
    .set(&key, &delivery);

env.storage()
    .persistent()
    .extend_ttl(&key, 518400, 518400);
```

The driver field is an `Option<Address>`.

After a successful assignment, it contains `Some(driver)`.

## 5. Event Emission

The contract publishes the `driver_assigned` event:

```rust
env.events().publish(
    (Symbol::new(&env, "driver_assigned"),),
    (delivery_id, driver),
);
```

### Event Details

| Event component | Value |
|---|---|
| Event topic | `driver_assigned` |
| First event value | `delivery_id` |
| Second event value | `driver` |

The event allows off-chain services and indexers to detect driver assignments and update their delivery read models.

## 6. Complete Assignment Flow

```text
Assignment Request
       |
       v
Require Caller Authorization
       |
       v
Check Administrator or Self-Assignment
       |
       v
Load Delivery Record
       |
       v
Validate Pending -> Active
       |
       v
Set Assigned Driver
       |
       v
Update Delivery Status
       |
       v
Persist Updated Record
       |
       v
Emit driver_assigned Event
```

Any failed authorization, missing delivery record, or invalid state transition prevents the assignment from completing.

## 7. Error and Rejection Conditions

| Condition | Result |
|---|---|
| Caller does not authorize the transaction | Authorization failure |
| Caller is neither admin nor assigned driver | `NotAuthorized` |
| Delivery identifier does not exist | `DeliveryNotFound` |
| Delivery is not in a valid assignment state | `InvalidState` |

The authorization and state failures currently use panic messages in the delivery contract rather than the `DeliveryError` enum.

## 8. Testing References

Driver assignment behavior is covered in:

`contracts/delivery_contract/test.rs`

Relevant tests include:

- `test_happy_path_full_lifecycle`
- `test_cancellation_after_assign`
- `test_invalid_assign_when_delivered`
- `test_unauthorized_assign_driver`

These tests cover successful assignment, assignment within a broader delivery lifecycle, invalid state rejection, and unauthorized assignment.

## 9. Source References

- `contracts/delivery_contract/lib.rs`
- `contracts/delivery_contract/test.rs`
- `contracts/shared_types/lib.rs`
- `docs/architecture/smart-contract-architecture.md`
