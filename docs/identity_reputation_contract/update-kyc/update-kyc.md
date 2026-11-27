
# Driver KYC Status Management

## Overview

The `update_driver_kyc_status` function allows the administrator of the SwiftChain identity and reputation contract to update a registered driver's Know Your Customer (KYC) verification status.

It modifies the driver's `kyc_verified` field, persists the updated profile, extends its storage time-to-live, and emits an event describing the status change.

**Contract:** `IdentityReputationContract`

**Implementation:** `contracts/identity_reputation_contract/lib.rs`

## Function Signature

```rust
pub fn update_driver_kyc_status(
    env: Env,
    admin: Address,
    driver: Address,
    kyc_verified: bool,
)
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment. |
| `admin` | `Address` | Address claiming to perform the administrator operation. |
| `driver` | `Address` | Registered driver's address. |
| `kyc_verified` | `bool` | New KYC verification status. |

The function does not return a value.

## Administrator Authorization

Updating KYC status is restricted to the administrator configured in the contract.

First, the supplied administrator must authorize the transaction:

```rust
admin.require_auth();
```

The contract then retrieves the stored administrator address:

```rust
let stored_admin: Address = env
    .storage()
    .instance()
    .get(&DataKey::Admin)
    .unwrap_or_else(|| {
        panic_with_error!(&env, SwiftChainError::NotInitialized)
    });
```

It compares the supplied address against the stored administrator:

```rust
if admin != stored_admin {
    panic_with_error!(&env, SwiftChainError::Unauthorized);
}
```

Both conditions are important:

1. The administrator must authorize the operation.
2. The administrator address must match the address stored during contract initialization.

Possessing a valid Stellar address or being a registered driver does not grant administrator privileges.

## KYC Boolean Semantics

The `kyc_verified` argument determines the driver's updated verification status.

| Value | Meaning |
|---|---|
| `true` | The driver's KYC status is marked as verified. |
| `false` | The driver's KYC status is marked as unverified. |

The value is stored directly in the driver's `DriverProfile`.

```rust
profile.kyc_verified = kyc_verified;
```

The function accepts either boolean value, allowing the administrator to mark a driver as verified or revert the status to unverified.

## Updating the Driver Profile

After administrator authorization, the contract:

1. Creates a driver profile storage key.
2. Retrieves the existing driver profile.
3. Updates its `kyc_verified` field.
4. Saves the modified profile.
5. Extends the profile's persistent storage time-to-live.
6. Publishes the KYC status update event.

The profile retrieval operation uses:

```rust
let key = DataKey::DriverProfile(driver.clone());

let mut profile: DriverProfile = env
    .storage()
    .persistent()
    .get(&key)
    .unwrap_or_else(|| {
        panic_with_error!(&env, SwiftChainError::ProviderNotFound)
    });
```

A driver must already have a registered profile before their KYC status can be updated.

## Downstream Tier and Eligibility Implications

The KYC flag is part of the driver's identity profile. However, it is important to distinguish KYC verification from reputation-based tiering and enterprise eligibility.

Under the current contract implementation:

- `get_driver_tier` calculates the driver's tier using `reputation_score`.
- `is_eligible_for_enterprise` checks whether `reputation_score` meets the enterprise threshold.
- Neither function checks `kyc_verified`.

Consequently, changing a driver's KYC status does not directly change their reputation score, driver tier, or enterprise eligibility result.

KYC verification is therefore an independent profile attribute in the current implementation.

Any future requirement that enterprise eligibility or delivery assignment must also depend on KYC status would require an explicit contract logic change.

## KYC Update Event

After successfully updating the profile, the contract publishes:

```rust
env.events().publish(
    (Symbol::new(&env, "kyc_status_updated"),),
    (driver, kyc_verified),
);
```

**Event name:** `kyc_status_updated`

**Event data:**

| Field | Description |
|---|---|
| `driver` | Address of the driver whose KYC status changed. |
| `kyc_verified` | New verification status. |

Off-chain services can monitor this event to identify KYC status changes.

## Error Handling

| Error | Condition |
|---|---|
| `NotInitialized` | No administrator has been configured in contract storage. |
| `Unauthorized` | The supplied administrator does not match the stored administrator. |
| `ProviderNotFound` | The specified driver profile does not exist. |

The error definitions are available in:

`contracts/shared_types/lib.rs`

## Testing References

Relevant tests are located in:

`contracts/identity_reputation_contract/test.rs`

- `test_kyc_status_update_by_admin`
- `test_kyc_status_update_unauthorized`
- `test_profile_fields_persisted`

These tests cover successful KYC updates, unauthorized administrator attempts, and profile persistence.

## Related Contract Functions

- `initialize`: Configures the contract administrator and related contract addresses.
- `get_admin`: Retrieves the configured administrator.
- `get_driver_profile`: Retrieves the driver's current profile.
- `get_driver_tier`: Calculates the driver's reputation-based tier.
- `is_eligible_for_enterprise`: Checks the reputation-based enterprise threshold.

## Implementation Reference

`contracts/identity_reputation_contract/lib.rs`
