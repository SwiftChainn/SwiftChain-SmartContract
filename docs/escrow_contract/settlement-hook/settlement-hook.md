
# Escrow Settlement Contract Hook

## Overview

The escrow contract supports optional integration with a settlement contract to enable asset conversion before driver payouts.

The integration allows a driver to receive a preferred asset instead of necessarily receiving the original token deposited into escrow.

The integration is configured through the escrow contract's settlement-contract setter and queried through its getter.

**Implementation reference:** `contracts/escrow_contract/lib.rs`

**Related architecture:** `docs/architecture/smart-contract-architecture.md`

## 1. Settlement Contract Configuration

The escrow contract maintains an optional settlement contract address in instance storage.

The storage key is defined internally as:

```rust
#[contracttype]
#[derive(Clone)]
enum DataKey {
    PendingAdmin,
    SettlementContract,
}
```

The settlement contract address is stored using `DataKey::SettlementContract`.

### 1.1 Setting the Settlement Contract

Function:

```rust
pub fn set_settlement_contract(
    env: Env,
    admin: Address,
    settlement_contract: Address,
)
```

Parameters:

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment. |
| `admin` | `Address` | Administrator configuring the integration. |
| `settlement_contract` | `Address` | Address of the settlement contract. |

Implementation:

```rust
pub fn set_settlement_contract(
    env: Env,
    admin: Address,
    settlement_contract: Address,
) {
    admin.require_auth();
    require_admin(&env, &admin);

    env.storage()
        .instance()
        .set(&DataKey::SettlementContract, &settlement_contract);
}
```

#### Authorization

Two authorization checks are performed:

1. `admin.require_auth()` requires the administrator's Soroban authorization.
2. `require_admin(&env, &admin)` compares the supplied administrator address against the administrator stored during contract initialization.

If the supplied address is not the registered administrator, the contract raises `SwiftChainError::Unauthorized`.

If the contract has not been initialized, the authorization helper raises `SwiftChainError::NotInitialized`.

Only the registered administrator can configure the settlement integration.

### 1.2 Retrieving the Settlement Contract

Function:

```rust
pub fn get_settlement_contract(env: Env) -> Option<Address>
```

The getter reads the configured address from instance storage.

Implementation:

```rust
pub fn get_settlement_contract(env: Env) -> Option<Address> {
    get_settlement_contract(&env)
}
```

The internal helper is:

```rust
fn get_settlement_contract(env: &Env) -> Option<Address> {
    env.storage()
        .instance()
        .get(&DataKey::SettlementContract)
}
```

The return type is `Option<Address>`.

- `Some(address)` means a settlement contract has been configured.
- `None` means no settlement contract address is currently stored.

The getter does not require administrator authorization.

## 2. How Settlement Is Triggered

Settlement routing is integrated into the internal `payout_driver` helper.

The escrow contract checks for a configured settlement contract whenever it processes a driver payout.

The relevant implementation is:

```rust
fn payout_driver(
    env: &Env,
    token: &Address,
    driver: &Address,
    amount: i128,
) {
    if amount <= 0 {
        return;
    }

    if let Some(settlement_addr) = get_settlement_contract(env) {
        let preferred_asset: Option<Address> = env.invoke_contract(
            &settlement_addr,
            &Symbol::new(env, "get_driver_preference"),
            soroban_sdk::vec![env, driver.into_val(env)],
        );

        if let Some(preferred_asset) = preferred_asset {
            if preferred_asset != token.clone() {
                let _: () = env.invoke_contract(
                    &settlement_addr,
                    &Symbol::new(env, "execute_settlement_swap"),
                    soroban_sdk::vec![
                        env,
                        env.current_contract_address().into_val(env),
                        token.into_val(env),
                        preferred_asset.into_val(env),
                        driver.into_val(env),
                        amount.into_val(env),
                        0i128.into_val(env),
                    ],
                );

                return;
            }
        }
    }

    token::Client::new(env, token)
        .transfer(&env.current_contract_address(), driver, &amount);
}
```

### 2.1 Settlement Routing Conditions

Swap routing occurs only when all the following conditions are satisfied:

1. The payout amount is greater than zero.
2. A settlement contract address has been configured.
3. The settlement contract returns a driver asset preference.
4. The preferred asset differs from the original escrow token.

When these conditions are satisfied, the escrow contract invokes `execute_settlement_swap`.

### 2.2 When Swap Routing Does Not Occur

The escrow contract transfers the original token directly to the driver when:

- No settlement contract is configured.
- The driver has not registered an asset preference.
- The driver's preferred asset is the same as the escrow token.

A non-positive payout amount also returns immediately without executing a transfer or swap.

### 2.3 Settlement Contract Invocation

The escrow contract invokes two settlement contract functions:

| Function | Purpose |
|---|---|
| `get_driver_preference` | Retrieves the driver's preferred payout asset. |
| `execute_settlement_swap` | Executes the conversion and pays the driver in the preferred asset. |

The swap invocation supplies:

| Argument | Value |
|---|---|
| Caller | Current escrow contract address |
| Source asset | Original escrow token |
| Destination asset | Driver's preferred asset |
| Recipient | Driver address |
| Amount | Calculated driver payout |
| Minimum amount out | `0` |

The settlement contract independently verifies that the caller is the configured escrow contract.

It also checks supported asset pairs, positive amounts, and the minimum output constraint.

The settlement contract uses its configured router for cross-asset conversion.

If the router returns an output below the requested minimum, its implementation falls back to paying the original asset.

The settlement contract emits `settlement_swap_executed` for successful cross-asset swaps.

## 3. Escrow Lifecycle Integration

The payout helper is called from the following escrow lifecycle operations:

### 3.1 Normal Escrow Release

`release_escrow`:

1. Requires caller authorization.
2. Checks that the caller is either the recipient or registered administrator.
3. Requires the escrow to be in the `Locked` state.
4. Verifies that the escrow contract has sufficient token balance.
5. Calculates the platform fee and driver's payout.
6. Calls `payout_driver`.
7. Transfers any platform fee to the administrator.
8. Changes the escrow status to `Released`.
9. Publishes the escrow release event.

### 3.2 Dispute Resolution

`resolve_dispute` also calls `payout_driver` when the administrator chooses to release the disputed escrow to the driver.

Therefore, settlement routing can occur during both normal escrow release and dispute resolution that results in driver payment.

Refund operations do not use `payout_driver`.

## 4. Relevant Events

The escrow contract publishes:

| Event | Description |
|---|---|
| `escrow_released` | Indicates that an escrow payout has been processed. |
| `dispute_resolved` | Indicates that an escrow dispute has been resolved. |

The settlement contract publishes:

| Event | Description |
|---|---|
| `settlement_swap_executed` | Records a successful cross-asset settlement swap. |

The escrow release event is published after the payout helper completes.

## 5. Security Considerations

- Only the stored administrator can configure the settlement contract.
- The administrator must provide valid Soroban authorization.
- The settlement contract validates that the caller is the configured escrow contract.
- Driver preferences are registered through the driver's authorized address.
- Supported asset pairs are checked by the settlement contract.
- Swap amounts must be positive.
- Settlement execution enforces its configured minimum-output requirement.

**Important:** The escrow currently passes `0` as `min_amount_out`. This means the escrow-to-settlement integration itself does not specify a positive minimum output. Do not describe this as a non-zero slippage guarantee.

## 6. Source References

- `contracts/escrow_contract/lib.rs`
- `contracts/settlement_contract/lib.rs`
- `contracts/shared_types/lib.rs`
- `docs/architecture/smart-contract-architecture.md`
