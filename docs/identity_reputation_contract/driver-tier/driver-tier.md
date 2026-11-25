
# Driver Tier System

## Overview

The SwiftChain identity and reputation contract implements a three-level driver tier system based on each driver's reputation score.

The system categorizes drivers into:

1. Bronze
2. Silver
3. Gold

A driver's tier is calculated dynamically using the current reputation score stored in their `DriverProfile`.

**Contract:** `IdentityReputationContract`

**Implementation:** `contracts/identity_reputation_contract/lib.rs`

## DriverTier Enum

The contract defines the `DriverTier` enum as follows:

```rust
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DriverTier {
    Bronze,
    Silver,
    Gold,
}
```

The enum represents the three possible driver reputation tiers.

These values are returned by `get_driver_tier`.

## Reputation Thresholds

The tier system uses the following reputation boundaries:

| Driver tier | Reputation score | Description |
|---|---:|---|
| Bronze | 0–49 | Drivers with a score below 50. |
| Silver | 50–74 | Drivers with a score of at least 50 but below 75. |
| Gold | 75–100 | Drivers with a score of at least 75. |

### Threshold Constants

The contract defines the enterprise eligibility threshold as:

```rust
const ENTERPRISE_THRESHOLD: u32 = 75;
```

The maximum reputation score is:

```rust
const MAX_REPUTATION: u32 = 100;
```

The tier calculation uses the reputation score directly.

## get_driver_tier Function

### Function Signature

```rust
pub fn get_driver_tier(
    env: Env,
    driver: Address,
) -> DriverTier
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment. |
| `driver` | `Address` | Driver whose tier should be retrieved. |

### Return Value

Returns one of the following `DriverTier` variants:

- `DriverTier::Bronze`
- `DriverTier::Silver`
- `DriverTier::Gold`

### Mapping Logic

The function first retrieves the driver's profile using:

```rust
let profile = Self::get_driver_profile(env, driver);
```

It then reads the driver's reputation score:

```rust
let score = profile.reputation_score;
```

The score is evaluated against the tier thresholds:

```rust
if score >= 75 {
    DriverTier::Gold
} else if score >= 50 {
    DriverTier::Silver
} else {
    DriverTier::Bronze
}
```

This means:

- A score of 75 or higher returns `Gold`.
- A score of 50 through 74 returns `Silver`.
- A score below 50 returns `Bronze`.

The contract does not store a separate tier value in `DriverProfile`. It calculates the tier whenever `get_driver_tier` is called.

## Tier Boundary Examples

| Reputation score | Returned tier |
|---:|---|
| 0 | Bronze |
| 25 | Bronze |
| 49 | Bronze |
| 50 | Silver |
| 60 | Silver |
| 74 | Silver |
| 75 | Gold |
| 90 | Gold |
| 100 | Gold |

The boundaries are inclusive at 50 for Silver and 75 for Gold.

## Initial Driver Tier

Newly registered drivers receive an initial reputation score of 50.

The registration function initializes:

```rust
reputation_score: 50,
```

Consequently, a newly registered driver starts in the Silver tier.

The initial tier is not assigned or persisted during registration. It is derived when `get_driver_tier` is called.

## Enterprise Eligibility

The contract also exposes:

```rust
pub fn is_eligible_for_enterprise(
    env: Env,
    driver: Address,
) -> bool
```

This function determines whether a driver meets the enterprise reputation threshold.

Its implementation is:

```rust
pub fn is_eligible_for_enterprise(
    env: Env,
    driver: Address,
) -> bool {
    let profile = Self::get_driver_profile(env, driver);

    profile.reputation_score >= ENTERPRISE_THRESHOLD
}
```

The function returns:

| Reputation score | Enterprise eligibility |
|---:|---|
| 0–74 | `false` |
| 75–100 | `true` |

A driver must therefore have a reputation score of at least 75 to satisfy the current enterprise eligibility check.

**Important:** Enterprise eligibility is a separate boolean query. The contract does not automatically persist an enterprise eligibility flag in the driver's profile.

## Relationship Between KYC and Tiering

The driver's KYC status is stored independently in:

```rust
pub kyc_verified: bool,
```

The current implementations of `get_driver_tier` and `is_eligible_for_enterprise` do not inspect this field.

Therefore:

- KYC verification does not automatically promote a driver.
- KYC verification does not automatically reduce a driver's tier.
- Enterprise eligibility is determined by reputation score alone.
- A driver with a reputation score of 75 or higher satisfies the current enterprise reputation threshold regardless of their KYC boolean value.

Any future requirement to include KYC verification in enterprise eligibility must be implemented explicitly in the contract.

## Reputation Changes and Tier Transitions

The driver's tier can change when their reputation score changes.

The relevant mutation functions are:

- `increase_reputation`
- `decrease_reputation`

For example:

| Previous score | Operation | New score | Resulting tier |
|---:|---|---:|---|
| 50 | Increase by 5 | 55 | Silver |
| 70 | Increase by 5 | 75 | Gold |
| 75 | Decrease by 10 | 65 | Silver |
| 50 | Decrease by 15 | 35 | Bronze |
| 95 | Increase by 10 | 100 | Gold |

These changes are reflected the next time the tier is queried.

## Error Handling

The tier and enterprise eligibility functions retrieve the driver's profile through `get_driver_profile`.

If the driver does not have an existing profile, the contract fails with:

`SwiftChainError::ProviderNotFound`

## Testing References

The relevant tests are located in:

`contracts/identity_reputation_contract/test.rs`

Tests covering tiering and reputation boundaries include:

- `test_tier_bronze`
- `test_tier_silver`
- `test_tier_gold`
- `test_tier_boundary_exact`
- `test_reputation_accumulation`
- `test_reputation_deduction_sequence`

These tests validate the tier mapping and the relationship between reputation changes and tier transitions.

## Related Contract Functions

- `register_driver`: Creates the driver's initial reputation profile.
- `get_driver_profile`: Retrieves the current driver profile.
- `increase_reputation`: Increases the driver's reputation.
- `decrease_reputation`: Decreases the driver's reputation.
- `is_eligible_for_enterprise`: Evaluates the enterprise reputation threshold.

## Implementation Reference

`contracts/identity_reputation_contract/lib.rs`
