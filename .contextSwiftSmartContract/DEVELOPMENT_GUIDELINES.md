# SwiftChain Development Guidelines

This document outlines the strict engineering rules for human contributors and AI agents working on the SwiftChain codebase.

## 1. Coding Standards (Smart Contracts)
- **Rust & Soroban**: All contracts MUST be `#![no_std]`. Avoid using the standard library. Use `soroban_sdk::Vec`, `soroban_sdk::String`, and `soroban_sdk::Map` instead of standard Rust collections.
- **Formatting**: Code must pass `cargo fmt` and `cargo clippy` without warnings.
- **Authorization**: EVERY state-mutating function MUST begin with a cryptographic authorization check. Use `address.require_auth()` for users, or verify admin status using stored data.
- **TTL Management**: Every time a persistent storage key is read from or written to, its Time-to-Live (TTL) MUST be extended using `env.storage().persistent().extend_ttl(&key, threshold, extend_to)`.

## 2. Architectural Rules
- **The `shared_types` Rule**: To prevent circular dependencies, any `Struct`, `Enum`, or `Error` that is passed between two different contracts MUST be defined in the `shared_types` crate. 
- **Cross-Contract Calls**: Use `env.invoke_contract()` for inter-contract communication. Be extremely careful with function signatures. If you change a function in Contract A, you must manually update the `invoke_contract` arguments in Contract B.
- **Event Emission**: Smart contracts do NOT serve complex queries. You MUST emit a detailed event (`env.events().publish(...)`) for every single state change. The off-chain indexer relies entirely on these events.

## 3. Error Handling Conventions
- **No Raw Panics**: Avoid using `panic!("string")`.
- **Custom Errors**: Always use custom `#[contracterror]` enums defined in `shared_types` (e.g., `SwiftChainError`, `EscrowError`).
- **Throwing Errors**: Use the `panic_with_error!(&env, ErrorEnum::Variant)` macro. This ensures the frontend and backend receive a specific error code rather than a generic trap.

## 4. State Machine Enforcement
- **Validation**: State transitions (e.g., in `delivery_contract`) must be strictly validated. A delivery cannot go from `Pending` directly to `Delivered`. It must pass through `Active` and `InTransit`.
- **Reentrancy/Order of Operations**: When doing cross-contract calls that transfer assets, update local state BEFORE making the external call to prevent reentrancy attacks, or rely on Soroban's native protections.

## 5. Testing Expectations
- **Coverage**: Every contract must have a companion `test.rs` file.
- **Mocking Auth**: Use `env.mock_all_auths()` to simulate transaction signatures in tests.
- **Failure Testing**: You MUST write tests that explicitly check for `Unauthorized` and `InvalidState` errors using `#[should_panic(expected = "...")]`.
- **Integration**: Write tests that deploy multiple contracts in the same test environment to verify cross-contract invocations work correctly.

## 6. File Organization Standards
- Each contract is its own crate inside the `contracts/` directory.
- `lib.rs`: Contains the contract logic, types specific ONLY to that contract, and the `DataKey` enum for storage.
- `test.rs`: Contains all unit tests.
- Constants (like TTL values, max reputation) should be grouped in a `pub mod constants { ... }` block at the top of the file.

## 7. Backend & Frontend Integration Rules (Off-Chain)
- **Do not bloat contract state**: Do not store large strings (like item descriptions or user bios) on-chain if they can be hashed or stored off-chain. The `DeliveryMetadata` struct should be kept as small as possible.
- **Read from Indexer**: The frontend should rarely read state directly from the RPC node. It should fetch complex views from the Node.js MongoDB backend.

## 8. Commit Conventions
Follow Conventional Commits:
- `feat: added fleet treasury routing`
- `fix: resolved unauthorized panic in escrow`
- `refactor: moved dispute errors to shared_types`
- `test: added e2e integration tests for delivery lifecycle`
