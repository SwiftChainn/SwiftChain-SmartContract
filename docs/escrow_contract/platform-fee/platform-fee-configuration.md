
# Platform Fee Configuration and Admin Updates

## Overview

The SwiftChain escrow contract supports configurable platform fees.

The platform fee is deducted from the escrow amount when funds are released to the driver. The deducted fee is transferred to the current contract administrator.

Fee configuration is maintained through the `ProtocolConfig` structure in `shared_types`.

The implementation is located in:

`contracts/escrow_contract/lib.rs`

## 1. Platform Fee Configuration

The contract stores platform fee configuration in:

```rust
pub struct ProtocolConfig {
    pub token: Address,
    pub platform_fee_bps: u32,
    pub protocol_version: u32,
}
```

The `platform_fee_bps` field represents the platform fee in basis points.

The configuration is stored using:

```rust
StorageKey::ProtocolConfig
```

The contract initializes the configuration through `init`.

## 2. Understanding Basis Points (bps)

A basis point is one-hundredth of one percent.

Therefore:

- 1 bps = 0.01%
- 100 bps = 1%
- 250 bps = 2.5%
- 500 bps = 5%
- 1,000 bps = 10%

The contract uses 10,000 as the denominator when calculating fees.

### Fee calculation

The implementation is:

```rust
fn calculate_fee(amount: i128, platform_fee_bps: u32) -> i128 {
    amount.saturating_mul(platform_fee_bps as i128) / 10_000
}
```

Mathematically:

```text
Platform Fee = (Escrow Amount × Fee in BPS) / 10,000
```

The division uses integer arithmetic, so fractional smallest-token-unit results are truncated.

### Example

Assume:

- Escrow amount = 10,000 token units
- Platform fee = 500 bps

Calculation:

```text
Platform Fee = (10,000 × 500) / 10,000

Platform Fee = 500 token units
```

The driver's payout is:

```text
Driver Payout = Escrow Amount - Platform Fee

Driver Payout = 10,000 - 500

Driver Payout = 9,500 token units
```

## 3. Updating the Platform Fee

The contract exposes:

```rust
pub fn update_platform_fee(
    env: Env,
    admin: Address,
    new_fee_bps: u32,
)
```

This operation updates the platform fee configuration.

### Authorization requirements

The supplied administrator address must match the administrator stored in:

```rust
StorageKey::Admin
```

The contract performs this comparison:

```rust
if admin != stored_admin {
    panic_with_error!(&env, SwiftChainError::Unauthorized);
}
```

It also requires the administrator to authorize the transaction:

```rust
admin.require_auth();
```

Both checks are important:

1. The supplied address must be the currently configured administrator.
2. The administrator must authorize the fee update.

An unauthorized account cannot update the platform fee.

## 4. Fee Validation

The contract rejects fee updates above 1,000 basis points.

```rust
if new_fee_bps > 1000 {
    panic_with_error!(&env, EscrowError::InvalidFee);
}
```

The current validation limit is:

```text
Maximum platform fee = 1,000 bps = 10%
```

Examples:

| Requested fee | Result |
|---|---|
| 0 bps | Accepted |
| 100 bps | Accepted |
| 250 bps | Accepted |
| 500 bps | Accepted |
| 1,000 bps | Accepted |
| 1,001 bps | Rejected |
| 1,500 bps | Rejected |

A zero fee is valid and disables platform fee deductions for subsequent applicable payouts.

**Implementation note:** The current maximum-fee validation is explicitly performed by `update_platform_fee`. The `init` function accepts its initial fee argument without applying the same 1,000-bps validation. Do not assume that initialization enforces this update-time limit.

## 5. Configuration Update Flow

The fee update follows this sequence:

```text
Administrator
     |
     v
update_platform_fee(admin, new_fee_bps)
     |
     v
Verify Stored Administrator
     |
     v
Require Administrator Authorization
     |
     v
Validate Fee <= 1,000 BPS
     |
     v
Load ProtocolConfig
     |
     v
Read Previous Fee
     |
     v
Update platform_fee_bps
     |
     v
Save ProtocolConfig
     |
     v
Emit FeeUpdated Event
```

If authorization or validation fails, the update is rejected.

## 6. FeeUpdated Event

After a successful update, the contract emits:

```rust
env.events().publish(
    (Symbol::new(&env, "FeeUpdated"),),
    FeeUpdated {
        old_fee,
        new_fee: new_fee_bps,
    },
);
```

The event payload is represented by:

```rust
pub struct FeeUpdated {
    pub old_fee: u32,
    pub new_fee: u32,
}
```

### Event structure

| Component | Value |
|---|---|
| Topic | `FeeUpdated` |
| Payload field 1 | `old_fee` |
| Payload field 2 | `new_fee` |

Both values are expressed in basis points.

External indexers can use this event to maintain an audit trail of platform fee changes.

## 7. Retrieving the Current Platform Fee

The contract exposes:

```rust
pub fn get_platform_fee(env: Env) -> u32
```

The function reads the current `ProtocolConfig` and returns:

```rust
load_protocol_config(&env).platform_fee_bps
```

This is a read-only contract operation.

Applications can use it to display the currently configured platform fee.

If the contract has not been initialized, loading the configuration fails with `SwiftChainError::NotInitialized`.

## 8. How Fees Are Applied During Escrow Release

When `release_escrow` is executed, the contract retrieves the configured fee and calculates the driver's payout.

```rust
let platform_fee = calculate_fee(record.amount, platform_fee_bps);
let driver_amount = record.amount.saturating_sub(platform_fee);
```

The driver receives the remaining amount.

The contract then transfers the platform fee to the administrator:

```rust
if platform_fee > 0 {
    let admin: Address = env
        .storage()
        .instance()
        .get(&StorageKey::Admin)
        .expect("Not initialized");

    token::Client::new(&env, &record.token).transfer(
        &env.current_contract_address(),
        &admin,
        &platform_fee,
    );
}
```

The platform fee is only transferred when it is greater than zero.

The same fee calculation is also applied when an administrator resolves a dispute by releasing funds to the driver through `resolve_dispute`.

Refunds through `refund_escrow` return the full recorded escrow amount to the sender without deducting the platform fee.

## 9. Fee Configuration Tests

Relevant tests are available in:

`contracts/escrow_contract/test.rs`

The current test suite includes:

| Test | Purpose |
|---|---|
| `test_init_and_platform_fee_default` | Verifies initial fee retrieval |
| `test_update_platform_fee_success` | Verifies successful fee update |
| `test_update_platform_fee_invalid_value` | Verifies rejection above the allowed maximum |
| `test_release_escrow_by_recipient_with_platform_fee_split` | Verifies payout and fee distribution |

These tests help confirm that the fee configuration and payout calculations behave as expected.

## 10. Security Considerations

- Only the stored administrator can update the platform fee.
- Administrator authorization is explicitly required.
- Fee updates above 1,000 bps are rejected.
- Fee changes are persisted in contract instance storage.
- Successful fee updates emit `FeeUpdated`.
- The administrator receiving fees is read from the current admin storage value during payout.
- Fee calculation uses integer arithmetic, so fractional token units are truncated.
- The initial fee passed to `init` is not subject to the update-time maximum check in the current implementation.

## Source References

- `contracts/escrow_contract/lib.rs`
- `contracts/escrow_contract/test.rs`
- `contracts/shared_types/lib.rs`
- `docs/architecture/smart-contract-architecture.md`
