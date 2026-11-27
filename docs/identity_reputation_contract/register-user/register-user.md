
# register_user Function

## Overview

The `register_user` function registers a new user in the SwiftChain Identity and Reputation Contract.

It creates a `UserProfile`, stores the profile in persistent contract storage, extends the storage TTL, and emits a registration event.

The function is implemented in:

```text
contracts/identity_reputation_contract/lib.rs
```

## 1. Function Signature

```rust
pub fn register_user(env: Env, user: Address) -> UserProfile
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment used for authorization, ledger access, storage, and events. |
| `user` | `Address` | Address of the account being registered. |

### Return Type

```rust
UserProfile
```

The returned profile is the newly created user profile.

The identity/reputation contract defines this structure as:

```rust
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct UserProfile {
    pub address: Address,
    pub join_date: u64,
}
```

## 2. Authorization

The function requires authorization from the user being registered:

```rust
user.require_auth();
```

This means the supplied user address must authorize the contract invocation.

The function does not accept an administrator address as a substitute for the user's authorization.

If the required authorization is not provided, Soroban authorization enforcement prevents the registration from succeeding.

## 3. Profile Initialization and Default Values

The function obtains the current ledger timestamp:

```rust
let join_date = env.ledger().timestamp();
```

It then constructs the profile:

```rust
let profile = UserProfile {
    address: user.clone(),
    join_date,
};
```

### Initial Profile Values

| Field | Initial value | Description |
|---|---|---|
| `address` | Supplied `user` address | Identifies the registered account. |
| `join_date` | Current ledger timestamp | Records when registration was performed. |

The current implementation does not initialize:

- A user reputation score.
- A user tier.
- A user delivery count.
- A KYC verification field.

Those fields are not part of the identity contract's `UserProfile` structure.

Driver reputation and tier management are handled separately through `DriverProfile` and the driver reputation functions.

## 4. Storage Key

The function uses the user's address to construct the profile storage key:

```rust
let key = DataKey::UserProfile(user.clone());
```

The relevant key variant is:

```rust
UserProfile(Address)
```

This makes the user address the identifier for the stored profile.

## 5. Duplicate Registration and Idempotency

Before writing the profile, the function checks whether the key already exists:

```rust
if env.storage().persistent().has(&key) {
    panic_with_error!(&env, SwiftChainError::AlreadyInitialized);
}
```

If a profile already exists for the supplied address, registration fails with:

```rust
SwiftChainError::AlreadyInitialized
```

### Is register_user Idempotent?

No.

The current implementation is not idempotent for repeated registration calls.

Calling `register_user` more than once with the same user address does not return the existing profile or overwrite it. The second call fails with `AlreadyInitialized`.

### Does It Overwrite Existing Profiles?

No.

The duplicate check occurs before the storage write. Therefore, the function does not overwrite an existing user profile.

This protects the original registration record from being replaced by a subsequent registration call.

## 6. Persistent Storage and TTL

After the duplicate check succeeds, the function stores the profile:

```rust
env.storage().persistent().set(&key, &profile);
```

It then extends the persistent storage TTL:

```rust
env.storage().persistent().extend_ttl(&key, 518400, 518400);
```

The TTL arguments are ledger-based values. They should not be described as seconds or days.

## 7. Registration Event

After storing the profile, the function publishes:

```rust
env.events()
    .publish((Symbol::new(&env, "user_registered"),), (user,));
```

### Event Details

| Element | Value |
|---|---|
| Event topic | `user_registered` |
| Event data | Registered user's address |

The event allows off-chain services and indexing applications to observe successful user registrations.

An off-chain consumer can use the event to trigger operations such as:

- Updating a user directory.
- Refreshing an application-side user read model.
- Recording registration activity.
- Triggering non-authoritative onboarding workflows.

The event should not be treated as a replacement for querying the stored profile.

## 8. Registration Flow

The current function follows this sequence:

```text
User invokes register_user
          |
          v
Require user authorization
          |
          v
Get current ledger timestamp
          |
          v
Construct UserProfile
          |
          v
Construct DataKey::UserProfile
          |
          v
Check whether profile exists
          |
          +---- Yes ----> Fail: AlreadyInitialized
          |
          No
          |
          v
Store profile in persistent storage
          |
          v
Extend storage TTL
          |
          v
Publish user_registered event
          |
          v
Return UserProfile
```

## 9. Retrieving a Registered User

The contract provides:

```rust
pub fn get_user_profile(env: Env, user: Address) -> UserProfile
```

This function reads the profile using:

```rust
DataKey::UserProfile(user)
```

If the profile exists, it returns the stored `UserProfile`.

If the profile does not exist, it panics with:

```rust
SwiftChainError::ProviderNotFound
```

## 10. Important Implementation Notes

- The user must authorize their own registration.
- The user address is used as the profile storage key.
- `join_date` is generated from the current Soroban ledger timestamp.
- The function stores profiles in persistent storage.
- The function extends the profile storage TTL.
- Duplicate registration fails with `AlreadyInitialized`.
- Existing profiles are not overwritten.
- The function returns the newly created `UserProfile`.
- The function emits `user_registered` with the user's address.
- No reputation score or tier is created for a regular user by this function.

## 11. Source References

- `contracts/identity_reputation_contract/lib.rs`
- `contracts/identity_reputation_contract/test.rs`
- `contracts/shared_types/lib.rs`
- `docs/architecture/smart-contract-architecture.md`
