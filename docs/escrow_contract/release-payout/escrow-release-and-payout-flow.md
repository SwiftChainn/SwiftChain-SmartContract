
# SwiftChain Escrow Release and Payout Flow

## 1. Overview

The escrow contract is responsible for holding delivery funds and releasing them when an authorized caller requests a valid payout.

The main release entry point is:

```rust
pub fn release_escrow(env: Env, caller: Address, delivery_id: u64)
```

**Source:** `contracts/escrow_contract/lib.rs`

The release process performs the following operations:

1. Authenticates the supplied caller.
2. Loads the escrow record.
3. Verifies that the caller is authorized.
4. Confirms that the escrow is in the `Locked` state.
5. Checks that the contract holds sufficient token balance.
6. Calculates the configured escrow platform fee.
7. Routes the net payout through `payout_driver`.
8. Transfers the platform fee to the administrator when applicable.
9. Marks the escrow as `Released`.
10. Saves the updated escrow record and publishes the release event.

The implementation also supports an optional settlement-contract integration for converting the driver's payout into a preferred asset.

## 2. release_escrow caller requirements

### Function signature

```rust
pub fn release_escrow(env: Env, caller: Address, delivery_id: u64)
```

### Authentication

The function first requires authorization from the supplied caller:

```rust
caller.require_auth();
```

This means the caller must authorize the invocation through Soroban's authorization mechanism.

### Caller eligibility

The function loads the escrow record and checks whether the caller is either:

- The configured escrow administrator.
- The recipient stored in the escrow record.

The relevant logic is:

```rust
let admin_authorized = is_admin(&env, &caller);
let recipient_authorized = caller == record.recipient;

if !admin_authorized && !recipient_authorized {
    panic_with_error!(&env, SwiftChainError::Unauthorized);
}
```

A sender or driver is not automatically authorized to invoke `release_escrow` unless that address also satisfies one of the accepted conditions.

### Escrow state requirement

The escrow must be in the `Locked` state:

```rust
if record.status != EscrowStatus::Locked {
    panic_with_error!(&env, EscrowError::InvalidState);
}
```

An already released, refunded, or paused escrow cannot pass this release-state check.

## 3. Balance verification

Before payout, the contract reads its balance for the escrow token:

```rust
let contract_balance =
    token::Client::new(&env, &record.token)
        .balance(&env.current_contract_address());

if contract_balance < record.amount {
    panic_with_error!(&env, EscrowError::InsufficientFunds);
}
```

The balance check ensures that the escrow contract has at least the amount recorded in the escrow entry before proceeding with the release.

This is an aggregate token-balance check against the individual escrow amount. It is not a separate token account or segregated balance for every delivery.

## 4. Platform fee calculation

The escrow contract obtains the platform fee basis points from `ProtocolConfig`:

```rust
let platform_fee_bps: u32 = env
    .storage()
    .instance()
    .get::<_, ProtocolConfig>(&StorageKey::ProtocolConfig)
    .map(|config| config.platform_fee_bps)
    .unwrap_or(0);
```

If the configuration is unavailable through this lookup, the code defaults the fee basis points to zero.

The fee is calculated using:

```rust
fn calculate_fee(amount: i128, platform_fee_bps: u32) -> i128 {
    amount.saturating_mul(platform_fee_bps as i128) / 10_000
}
```

The calculation is:

```text
platform_fee = escrow_amount × platform_fee_bps / 10,000

driver_amount = escrow_amount - platform_fee
```

One basis point is 0.01%.

For example, if the escrow amount is 10,000 token units and the configured fee is 100 basis points:

```text
Platform fee = 10,000 × 100 / 10,000
             = 100

Net driver payout = 10,000 - 100
                  = 9,900
```

This is an illustrative calculation, not a hardcoded protocol fee.

## 5. Driver payout function

After calculating the fee, `release_escrow` calls:

```rust
payout_driver(
    &env,
    &record.token,
    &record.driver,
    driver_amount,
);
```

The helper is defined in `contracts/escrow_contract/lib.rs`.

Its behavior is:

1. Return without transferring if the payout amount is zero or negative.
2. Check whether a settlement contract has been configured.
3. If configured, query the driver's preferred settlement asset.
4. If the preferred asset exists and differs from the escrow token, invoke the settlement swap.
5. Otherwise, transfer the original escrow token directly to the driver.

The relevant settlement lookup is:

```rust
let preferred_asset: Option<Address> = env.invoke_contract(
    &settlement_addr,
    &Symbol::new(env, "get_driver_preference"),
    soroban_sdk::vec![env, driver.into_val(env)],
);
```

When the preferred asset differs from the escrow token, the escrow contract invokes:

```rust
execute_settlement_swap(
    escrow_contract_address,
    source_asset,
    preferred_asset,
    driver,
    amount,
    minimum_amount_out,
)
```

The current implementation supplies `0i128` as the minimum amount out.

If there is no configured settlement contract, no driver preference, or the preferred asset matches the escrow token, the helper performs a direct token transfer:

```rust
token::Client::new(env, token).transfer(
    &env.current_contract_address(),
    driver,
    &amount,
);
```

## 6. Settlement contract integration

The escrow contract stores an optional settlement contract address under its local `DataKey::SettlementContract`.

The settlement integration is therefore conditional.

### When settlement is not configured

The driver's net payout is transferred in the original escrow token.

### When settlement is configured

The escrow contract asks the settlement contract for the driver's preferred asset.

If the preferred asset differs from the original escrow token, the escrow contract calls `execute_settlement_swap`.

The settlement contract then handles the configured asset pair, router interaction, minimum-output validation, and its own settlement fee logic.

### Important distinction: two fee calculations

The current implementation has two fee-related stages:

1. **Escrow platform fee:** calculated by the escrow contract using `ProtocolConfig.platform_fee_bps`. This amount is deducted before `payout_driver` is called and is transferred to the escrow administrator.
2. **Settlement conversion fee:** the settlement contract has its own `PlatformFeeBps` configuration and applies that fee in its swap execution path.

These are separate configuration values and should not be described as one shared fee.

When a cross-asset conversion occurs, the settlement fee can affect the amount received by the driver in the preferred asset.

The escrow release event reports the escrow-level driver amount and escrow-level platform fee. It should not be interpreted as a complete report of any separate settlement conversion fee.

## 7. Driver versus fleet treasury payout routing

### Current escrow behavior

The current `release_escrow` implementation passes:

```rust
&record.driver
```

to `payout_driver`.

The payout helper then sends the payout to that driver address, either directly in the escrow token or through the settlement conversion path.

The current release function does not call the fleet-management contract's `get_payout_address` function.

Therefore, the existing `release_escrow` flow should be documented as driver-address payout routing.

### Fleet-management capability

The fleet-management contract defines:

```rust
pub fn get_payout_address(
    env: Env,
    driver: Address,
    fleet_id: FleetId,
) -> Address
```

The function checks the driver's membership status in the specified fleet.

If the driver is an active member, it loads the fleet profile and returns:

```rust
profile.treasury
```

Otherwise, it returns the driver's own address.

Conceptually:

```text
                         get_payout_address
                                  |
                                  v
                      Is driver an active member?
                           /             \
                         Yes              No
                          |                |
                          v                v
                   Fleet treasury     Driver address
```

### Integration status

Although the fleet-management contract exposes a payout-address lookup, the current escrow release implementation does not invoke it.

Consequently:

- The fleet treasury lookup exists.
- The active-driver membership check exists.
- The escrow release path currently pays `record.driver`.
- Automatic fleet treasury routing is not yet integrated into `release_escrow`.

This distinction is important because documenting an existing function is not the same as claiming that the function is already connected to the escrow payout flow.

A future integration would need to define how the escrow contract obtains the applicable fleet identifier, how the fleet-management contract is configured, and how the returned payout address is used safely with the settlement path.

That integration is outside the current documentation-only scope.

## 8. Platform fee transfer

After the driver payout helper completes, the escrow contract transfers the escrow-level platform fee to the configured administrator when the fee is greater than zero.

The relevant implementation is:

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

The administrator address is read from the shared `StorageKey::Admin`.

If the fee is zero, the fee transfer is skipped.

## 9. Escrow state update

After the payout and fee-transfer operations, the escrow record is updated:

```rust
record.status = EscrowStatus::Released;
save_escrow(&env, delivery_id, &record);
```

The `save_escrow` helper:

- Constructs the shared escrow key.
- Writes the updated record to persistent storage.
- Extends the record's TTL using the configured escrow TTL constants.

This ensures that the stored escrow state reflects the release operation.

## 10. Escrow release event

After saving the updated record, the contract publishes the release event:

```rust
env.events().publish(
    (events::escrow_released(&env), delivery_id),
    (record.driver, driver_amount, platform_fee),
);
```

The event contains:

| Event data | Meaning |
|---|---|
| Event topic | `escrow_released` |
| Delivery identifier | The delivery whose escrow was released. |
| Driver address | The driver address stored in the escrow record. |
| Driver amount | The amount passed to the payout helper after the escrow-level fee deduction. |
| Platform fee | The escrow-level platform fee deducted from the original escrow amount. |

An event consumer should distinguish the event's reported driver amount from the final preferred-asset amount received after a settlement conversion.

## 11. Release flow summary

```text
Caller invokes release_escrow
              |
              v
       Require caller auth
              |
              v
        Load escrow record
              |
              v
   Is caller admin or recipient?
          /          \
        No            Yes
        |              |
        v              v
  Unauthorized    Check escrow state
                       |
                       v
                Is status Locked?
                   /       \
                 No         Yes
                 |           |
                 v           v
             Invalid     Check token balance
              state            |
                               v
                      Calculate platform fee
                               |
                               v
                       Calculate net payout
                               |
                               v
                        payout_driver
                               |
                +--------------+--------------+
                |                             |
          Settlement path                Direct transfer
          if configured                  in escrow token
                |                             |
                +--------------+--------------+
                               |
                               v
                  Transfer escrow platform fee
                    to administrator if > 0
                               |
                               v
                    Set status to Released
                               |
                               v
                      Save escrow record
                               |
                               v
                    Publish escrow_released
```

## 12. Failure and consistency considerations

The release function uses typed errors for important validation failures:

| Condition | Error |
|---|---|
| Caller is not authorized | `SwiftChainError::Unauthorized` |
| Escrow is not locked | `EscrowError::InvalidState` |
| Contract token balance is insufficient | `EscrowError::InsufficientFunds` |
| Escrow record does not exist | `EscrowError::DeliveryNotFound` |

Soroban contract invocations execute transactionally. If a contract invocation fails, the transaction's state changes are rolled back according to Soroban transaction semantics.

The current source does not implement a separate application-level reentrancy guard in `release_escrow`. This document does not claim that one exists.

The current balance check compares the contract's aggregate token balance with the individual escrow amount. It should not be interpreted as segregated per-delivery accounting.

## 13. Contributor guidance

When changing the escrow release implementation:

1. Preserve caller authorization requirements.
2. Preserve the `Locked` state requirement unless the protocol state machine is deliberately changed.
3. Preserve balance verification before payout.
4. Keep escrow platform-fee calculation separate from settlement conversion fees.
5. Review the settlement contract interface whenever payout conversion behavior changes.
6. Do not claim fleet treasury routing until the escrow release path actually invokes the fleet-management payout lookup.
7. Keep the release event payload documentation synchronized with the actual publication code.
8. Update this document when payout routing, fee calculation, or authorization rules change.

## 14. Source references

- `contracts/escrow_contract/lib.rs`
  - `release_escrow`
  - `payout_driver`
  - `calculate_fee`
  - `save_escrow`
  - `load_escrow`
  - `get_settlement_contract`
- `contracts/fleet_management_contract/lib.rs`
  - `FleetProfile`
  - `get_payout_address`
- `contracts/settlement_contract/lib.rs`
  - `get_driver_preference`
  - `execute_settlement_swap`
  - `PlatformFeeBps`
- `contracts/shared_types/lib.rs`
  - `EscrowRecord`
  - `EscrowState`
  - `ProtocolConfig`
  - `StorageKey`
- `docs/architecture/smart-contract-architecture.md`
