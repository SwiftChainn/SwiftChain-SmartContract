
# Two-Step Admin Ownership Transfer

## Overview

The SwiftChain escrow contract implements a two-step administrator ownership transfer mechanism.

Instead of immediately replacing the current administrator, the contract requires:

1. The current administrator to propose a new administrator.
2. The proposed administrator to explicitly accept the transfer.

This mechanism reduces the risk of accidentally transferring administrative control to an incorrect or inaccessible address.

The implementation is located in:

`contracts/escrow_contract/lib.rs`

## 1. Administrative Storage

The current administrator is stored using:

```rust
StorageKey::Admin
```

The contract also defines a private storage key for the proposed administrator:

```rust
#[contracttype]
#[derive(Clone)]
enum DataKey {
    PendingAdmin,
    SettlementContract,
}
```

The `PendingAdmin` key stores the address that has been nominated to become the next administrator.

The current administrator remains unchanged until the proposed administrator successfully accepts the transfer.

## 2. Step One: Proposing a New Administrator

The contract exposes:

```rust
pub fn propose_admin(
    env: Env,
    current_admin: Address,
    new_admin: Address,
)
```

The function allows the current administrator to nominate a new administrator.

### Authorization

The current administrator must authorize the transaction:

```rust
current_admin.require_auth();
```

The contract retrieves the currently stored administrator:

```rust
let stored_admin: Address = env
    .storage()
    .instance()
    .get(&StorageKey::Admin)
    .unwrap_or_else(|| panic_with_error!(&env, SwiftChainError::NotInitialized));
```

It then verifies that the supplied address matches the stored administrator:

```rust
if stored_admin != current_admin {
    panic!("caller is not the admin");
}
```

If the address does not match, the proposal is rejected.

### Storing the proposed administrator

After successful authorization, the contract stores the proposed address:

```rust
env.storage()
    .instance()
    .set(&DataKey::PendingAdmin, &new_admin);
```

It also extends the instance storage TTL:

```rust
env.storage().instance().extend_ttl(
    constants::ESCROW_TTL_THRESHOLD,
    constants::ESCROW_TTL_EXTEND_TO,
);
```

The current implementation uses `518400` for both TTL parameters.

**Important:** Proposing a new administrator does not immediately change the administrator address.

The existing administrator retains administrative authority until the second step is completed.

## 3. Step Two: Accepting the Administrator Transfer

The proposed administrator must explicitly accept the nomination.

The contract exposes:

```rust
pub fn accept_admin(
    env: Env,
    new_admin: Address,
)
```

### Authorization

The proposed administrator must authorize the transaction:

```rust
new_admin.require_auth();
```

This ensures that the nominated account participates in accepting administrative control.

### Verify the pending administrator

The contract retrieves the pending administrator:

```rust
let pending: Address = env
    .storage()
    .instance()
    .get(&DataKey::PendingAdmin)
    .expect("no pending admin");
```

It verifies that the accepting address matches the stored proposal:

```rust
if pending != new_admin {
    panic!("caller is not the pending admin");
}
```

If the address does not match, the transfer is rejected.

### Update administrator ownership

Once the proposed administrator is verified, the contract retrieves the previous administrator:

```rust
let old_admin: Address = env
    .storage()
    .instance()
    .get(&StorageKey::Admin)
    .unwrap_or_else(|| panic_with_error!(&env, SwiftChainError::NotInitialized));
```

It then updates the administrator:

```rust
env.storage().instance().set(&StorageKey::Admin, &new_admin);
```

The pending administrator entry is removed:

```rust
env.storage().instance().remove(&DataKey::PendingAdmin);
```

Finally, the contract extends its instance storage TTL and emits an administrative transfer event.

## 4. AdminTransferred Event

A successful administrator transfer emits:

```rust
env.events().publish(
    (Symbol::new(&env, "AdminTransferred"),),
    (old_admin, new_admin),
);
```

### Event structure

| Component | Value |
|---|---|
| Topic | `AdminTransferred` |
| Payload field 1 | Previous administrator address |
| Payload field 2 | New administrator address |

The event provides an on-chain record of the completed administrator ownership transfer.

**Important:** The event is emitted after acceptance, not when the proposal is created.

The `propose_admin` function does not currently emit a dedicated event.

## 5. Complete Ownership Transfer Flow

The complete process is:

```text
             CURRENT ADMIN
                   |
                   |
                   v
          propose_admin(...)
                   |
                   v
        Verify Current Admin
                   |
                   v
        Require Current Admin Auth
                   |
                   v
       Store PendingAdmin Address
                   |
                   v
        WAIT FOR ACCEPTANCE
                   |
                   |
                   v
           PROPOSED ADMIN
                   |
                   v
            accept_admin(...)
                   |
                   v
        Require Proposed Admin Auth
                   |
                   v
        Verify PendingAdmin Address
                   |
                   v
         Update StorageKey::Admin
                   |
                   v
         Remove PendingAdmin Key
                   |
                   v
          Emit AdminTransferred
                   |
                   v
        OWNERSHIP TRANSFERRED
```

## 6. Why Use a Two-Step Transfer?

A single-step ownership transfer immediately changes the administrator to the supplied address.

Although straightforward, this approach creates risks if the supplied address is incorrect or cannot authorize transactions.

The two-step mechanism introduces an acceptance requirement.

### Security benefits

**1. Explicit acceptance**

The proposed administrator must authorize the acceptance transaction before ownership changes.

**2. Address verification**

The accepting address must match the address previously stored as `PendingAdmin`.

**3. Reduced accidental ownership transfers**

The current administrator cannot complete a transfer simply by supplying an address. The nominated account must participate.

**4. Clear ownership transition**

The `AdminTransferred` event identifies both the previous and new administrators.

**5. Preservation of current administrative authority**

The current administrator remains active while the proposal is pending.

## 7. Edge Cases and Expected Behavior

| Scenario | Expected behavior |
|---|---|
| Contract is not initialized | Proposal fails because no stored administrator exists |
| Non-admin attempts to propose a new admin | Proposal is rejected |
| Current admin does not authorize proposal | Authorization fails |
| Proposed admin does not authorize acceptance | Authorization fails |
| No pending administrator exists | Acceptance fails |
| Wrong address attempts acceptance | Acceptance is rejected |
| Correct proposed admin accepts | Ownership changes successfully |
| Ownership transfer completes | Pending administrator entry is removed |

### Additional implementation considerations

The current implementation has the following behaviors that integrators should understand:

- A subsequent valid proposal overwrites the existing `PendingAdmin` value.
- The previous administrator remains the active administrator until acceptance succeeds.
- There is no dedicated cancellation function for a pending proposal.
- There is no explicit time-based expiration check for a pending administrator.
- The contract extends instance storage TTL during both proposal and acceptance.
- The current implementation does not emit an event when a proposal is created.
- The new administrator must be able to authorize a Soroban transaction to accept ownership.

These behaviors should be considered when building administrative dashboards and monitoring tools.

## 8. Interaction with Other Admin-Only Functions

The administrator address is used by other escrow operations.

For example:

- `update_platform_fee` requires the current administrator.
- `set_settlement_contract` requires administrator authorization.
- `resolve_dispute` requires administrator authorization.
- `resolve_dispute_split` requires administrator authorization.
- `release_escrow` permits the administrator as an authorized caller.
- `refund_escrow` permits the administrator as an authorized caller.

After a successful transfer, these operations recognize the newly stored administrator address.

The old administrator no longer satisfies checks that compare the caller against `StorageKey::Admin`.

## 9. Integration Recommendations

Applications implementing administrator management should:

1. Display the current administrator address.
2. Require confirmation before submitting a proposal.
3. Display the proposed administrator address.
4. Communicate that the transfer is pending until accepted.
5. Require the proposed administrator to connect and authorize the acceptance transaction.
6. Listen for `AdminTransferred`.
7. Refresh the administrator information after successful acceptance.
8. Handle failed authorization and invalid pending-address attempts.

Applications should not report that administrator ownership has changed immediately after `propose_admin` succeeds.

Ownership changes only after `accept_admin` succeeds.

## Source References

- `contracts/escrow_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `contracts/escrow_contract/test.rs`
- `docs/architecture/smart-contract-architecture.md`
