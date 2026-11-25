
# Driver Reputation Management

## Overview

The SwiftChain identity and reputation contract maintains an on-chain reputation score for every registered driver.

The reputation system provides two primary mutation functions:

- `increase_reputation`: Rewards drivers through reputation increases following authorized delivery-related operations.
- `decrease_reputation`: Applies reputation deductions through authorized delivery or dispute-related operations.

Both functions update the driver's existing `DriverProfile` and emit events that can be monitored by off-chain applications.

**Contract:** `IdentityReputationContract`

**Implementation:** `contracts/identity_reputation_contract/lib.rs`

## Reputation Score Range

The contract defines the maximum reputation score as:

```rust
const MAX_REPUTATION: u32 = 100;
```

The intended reputation range is:

**0 to 100**

| Boundary | Meaning |
|---|---|
| `0` | Minimum reputation score. |
| `100` | Maximum reputation score. |

Newly registered drivers start with a reputation score of `50`.

## 1. Increasing Reputation

### Function Signature

```rust
pub fn increase_reputation(
    env: Env,
    caller: Address,
    driver: Address,
    delivery_id: u64,
    weight_grams: u32,
    fragile: bool,
)
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment. |
| `caller` | `Address` | Authorized delivery or dispute contract address. |
| `driver` | `Address` | Driver whose reputation will increase. |
| `delivery_id` | `u64` | Identifier of the associated delivery. |
| `weight_grams` | `u32` | Cargo weight in grams. |
| `fragile` | `bool` | Indicates whether the cargo is fragile. |

### Authorization

Reputation increases are restricted to the configured delivery and dispute contracts.

The contract retrieves both authorized addresses:

```rust
let delivery_contract: Address = env
    .storage()
    .instance()
    .get(&DataKey::DeliveryContract)
    .unwrap_or_else(|| {
        panic_with_error!(&env, SwiftChainError::NotInitialized)
    });

let dispute_contract: Address = env
    .storage()
    .instance()
    .get(&DataKey::DisputeContract)
    .unwrap_or_else(|| {
        panic_with_error!(&env, SwiftChainError::NotInitialized)
    });
```

The caller must match one of these addresses:

```rust
if caller != delivery_contract && caller != dispute_contract {
    panic_with_error!(&env, SwiftChainError::Unauthorized);
}
```

The authorized caller must also authorize the invocation:

```rust
caller.require_auth();
```

An arbitrary user or driver cannot directly increase their own reputation through this function.

### Reputation Calculation Rules

The function calculates reputation points using three conditions.

| Condition | Points awarded |
|---|---:|
| Base delivery reward | +5 |
| Cargo weight greater than 5,000 grams | +3 |
| Cargo marked as fragile | +2 |

The base reward is always applied.

The additional weight reward is applied only when:

```rust
weight_grams > 5000
```

A cargo weight of exactly 5,000 grams does not qualify for the additional three points.

The fragile reward is applied when:

```rust
fragile == true
```

### Calculation Implementation

```rust
let mut points: u32 = 5;

if weight_grams > 5000 {
    points += 3;
}

if fragile {
    points += 2;
}
```

The resulting reward ranges from 5 to 10 points per invocation.

### Reputation Ceiling

The contract prevents reputation from exceeding 100:

```rust
profile.reputation_score =
    (profile.reputation_score + points).min(MAX_REPUTATION);
```

The `min` operation ensures that any calculated score above the maximum is capped at 100.

### Delivery Count

Every successful reputation increase also increments the driver's completed-delivery counter:

```rust
profile.deliveries_completed += 1;
```

This increment occurs once per successful invocation of `increase_reputation`.

### Reputation Increase Event

The contract publishes:

```rust
env.events().publish(
    (Symbol::new(&env, "reputation_increased"),),
    (driver, delivery_id, points),
);
```

**Event name:** `reputation_increased`

**Event data:**

- `driver`: Driver address.
- `delivery_id`: Associated delivery identifier.
- `points`: Reputation points calculated for the operation.

## 2. Decreasing Reputation

### Function Signature

```rust
pub fn decrease_reputation(
    env: Env,
    caller: Address,
    driver: Address,
    points: u32,
)
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment. |
| `caller` | `Address` | Authorized delivery or dispute contract address. |
| `driver` | `Address` | Driver whose reputation will decrease. |
| `points` | `u32` | Number of reputation points to deduct. |

### Authorization

The authorization mechanism is identical to `increase_reputation`.

Only the configured delivery contract or dispute contract can perform the operation.

The supplied caller must:

1. Match one of the configured contract addresses.
2. Authorize the operation through `caller.require_auth()`.

Unauthorized callers cause the contract to fail with `SwiftChainError::Unauthorized`.

### Reputation Deduction

The contract uses saturating subtraction:

```rust
profile.reputation_score =
    profile.reputation_score.saturating_sub(points);
```

This prevents the reputation score from becoming negative or underflowing below zero.

For example:

| Previous score | Deduction | New score |
|---:|---:|---:|
| 50 | 10 | 40 |
| 50 | 50 | 0 |
| 50 | 80 | 0 |
| 75 | 15 | 60 |
| 100 | 20 | 80 |

If the requested deduction is greater than the current reputation score, the resulting score is zero.

### Reputation Decrease Event

After the profile is updated, the contract publishes:

```rust
env.events().publish(
    (Symbol::new(&env, "reputation_decreased"),),
    (driver, points),
);
```

**Event name:** `reputation_decreased`

**Event data:**

- `driver`: Driver address.
- `points`: Requested deduction amount.

The event records the requested deduction, not necessarily the actual reduction when the previous score is lower than the requested points.

## Storage Updates

Both reputation functions follow the same storage update pattern:

1. Retrieve the driver's existing profile.
2. Update the reputation-related fields.
3. Persist the updated profile.
4. Extend its persistent storage time-to-live.
5. Emit the corresponding event.

The storage operation uses the `DataKey::DriverProfile` key.

A missing driver profile results in `SwiftChainError::ProviderNotFound`.

## Important Behavioral Differences

| Behavior | Increase | Decrease |
|---|---|---|
| Authorized callers | Delivery or dispute contract | Delivery or dispute contract |
| Authorization | `caller.require_auth()` | `caller.require_auth()` |
| Score operation | Addition | Saturating subtraction |
| Score boundary | Maximum 100 | Minimum 0 |
| Updates completed deliveries | Yes | No |
| Emits event | `reputation_increased` | `reputation_decreased` |

Neither operation directly modifies the driver's KYC status.

## Relationship With Driver Tiering

Reputation changes can affect the driver's tier because `get_driver_tier` derives the tier from the current reputation score.

The enterprise eligibility check also uses the reputation score.

Therefore, increasing or decreasing reputation can change the result of these queries without requiring a separate tier update.

Refer to:

`docs/identity_reputation_contract/driver-tier/driver-tier.md`

## Testing References

Tests are located in:

`contracts/identity_reputation_contract/test.rs`

Relevant tests include:

- `test_increase_reputation_basic`
- `test_decrease_reputation_basic`
- `test_reputation_cannot_go_below_zero`
- `test_reputation_upper_bound`
- `test_reputation_accumulation`
- `test_reputation_deduction_sequence`

## Implementation Reference

`contracts/identity_reputation_contract/lib.rs`
