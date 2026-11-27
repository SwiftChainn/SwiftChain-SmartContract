
# Driver Registration

## Overview

The `register_driver` function registers a driver independently of the regular user-registration flow in the SwiftChain identity and reputation contract.

It creates an on-chain `DriverProfile`, initializes the driver's reputation and verification status, stores the profile in persistent contract storage, and emits a registration event.

**Contract:** `IdentityReputationContract`

**Implementation:** `contracts/identity_reputation_contract/lib.rs`

## Function Signature

```rust
pub fn register_driver(env: Env, driver: Address)
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment. |
| `driver` | `Address` | Stellar address of the driver being registered. |

The function does not return a value.

## Authorization

Driver registration requires authorization from the driver's own address.

```rust
driver.require_auth();
```

This invokes Soroban's authorization mechanism and requires the driver to authorize the registration operation.

There is no administrator authorization requirement for `register_driver`.

The driver address is also used as the key for the driver's profile in persistent storage.

## Registration Flow

The registration process follows these steps:

1. The driver authorizes the registration request.
2. The contract constructs a `DataKey::DriverProfile` using the driver's address.
3. The contract checks whether a profile already exists.
4. If the profile exists, registration fails with `SwiftChainError::AlreadyInitialized`.
5. If the profile does not exist, a new `DriverProfile` is created.
6. The profile is saved in persistent storage.
7. The contract extends the storage entry's time-to-live.
8. The contract emits the `driver_registered` event.

## Driver Profile Defaults

The contract initializes every newly registered driver with the following values:

| Field | Default | Description |
|---|---|---|
| `address` | Driver address | Identifies the registered driver. |
| `deliveries_completed` | `0` | Initial completed-delivery count. |
| `reputation_score` | `50` | Starting reputation score. |
| `registered_at` | Current ledger timestamp | Registration time. |
| `kyc_verified` | `false` | Driver has not yet been marked as KYC verified. |

The profile is represented by the `DriverProfile` struct:

```rust
pub struct DriverProfile {
    pub address: Address,
    pub deliveries_completed: u32,
    pub reputation_score: u32,
    pub registered_at: u64,
    pub kyc_verified: bool,
}
```

A newly registered driver therefore starts with a reputation score of 50 and an initial `Bronze`, `Silver`, or `Gold` tier determined by the tier thresholds. Under the current implementation, the initial score of 50 corresponds to `Silver`.

## Duplicate Registration

The contract prevents duplicate driver registration.

It checks persistent storage using:

```rust
let key = DataKey::DriverProfile(driver.clone());

if env.storage().persistent().has(&key) {
    panic_with_error!(&env, SwiftChainError::AlreadyInitialized);
}
```

If the driver already has a profile, the transaction fails with `AlreadyInitialized`.

This prevents an existing profile from being overwritten through another registration request.

## Storage and Time-to-Live

Driver profiles are stored in Soroban persistent storage.

After saving the profile, the contract calls:

```rust
env.storage()
    .persistent()
    .extend_ttl(&key, 518400, 518400);
```

This extends the storage entry's time-to-live using the specified ledger thresholds.

## Registration Event

After successful registration, the contract publishes:

```rust
env.events()
    .publish(
        (Symbol::new(&env, "driver_registered"),),
        (driver,),
    );
```

**Event name:** `driver_registered`

**Event data:** The registered driver's address.

This event allows external applications and off-chain indexers to detect successful driver registrations.

## Relationship With User Registration

Driver registration and user registration are separate operations.

| Function | Purpose | Profile |
|---|---|---|
| `register_driver` | Registers a delivery driver. | `DriverProfile` |
| `register_user` | Registers a regular platform user. | `UserProfile` |

The `register_user` function creates a `UserProfile` containing an address and join date. It does not initialize driver-specific reputation, delivery counts, or KYC information.

A regular user registration must therefore not be treated as an automatic driver registration.

## Related Contract Functions

- `get_driver_profile`: Retrieves an existing driver's profile.
- `get_driver_tier`: Determines a driver's tier from their reputation score.
- `update_driver_kyc_status`: Allows the administrator to update the driver's KYC status.
- `increase_reputation`: Updates reputation after an authorized delivery-related operation.
- `decrease_reputation`: Applies an authorized reputation deduction.

## Error Handling

| Error | Condition |
|---|---|
| `AlreadyInitialized` | The driver already has a registered profile. |

The error definitions are maintained in `contracts/shared_types/lib.rs`.

## Testing References

Registration behavior is covered in:

`contracts/identity_reputation_contract/test.rs`

Relevant tests include:

- `test_register_driver`
- `test_register_driver_duplicate`
- `test_profile_fields_persisted`

## Implementation Reference

`contracts/identity_reputation_contract/lib.rs`
