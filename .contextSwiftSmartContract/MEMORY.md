# SwiftChain Project Memory

## Project Overview
**Purpose**: SwiftChain is a Blockchain-Powered Logistics & Escrow Delivery Platform designed to create a shared, decentralized logistics economy. It connects individuals and businesses with independent transport providers (motorcycle riders, vans, truck operators).
**Business Goal**: Solve the lack of trust in traditional logistics, eliminate high fees, and ensure instant settlements for drivers. It aims to drive financial inclusion for independent drivers and small logistics businesses.
**Target Users**: African logistics networks, SME merchants, e-commerce sellers, courier startups, transport unions, and cross-border trade operators.

## Tech Stack
- **Smart Contracts (This Repo)**: Stellar Soroban, Rust, Soroban SDK, WASM.
- **Frontend (Separate Repo)**: Next.js, TypeScript, TailwindCSS.
- **Backend (Separate Repo)**: Node.js, Express.js, TypeScript, MongoDB.
- **Infrastructure**: Stellar Testnet/Mainnet, GitHub Actions (CI/CD).
- **Package Managers**: Cargo (Rust), Make/Bash scripts.

## System Architecture
SwiftChain operates on a hybrid on-chain/off-chain architecture:
- **On-Chain**: 7 core modular smart contracts govern the absolute source of truth for financial escrows, delivery states, identity/reputation, and fleet management.
- **Off-Chain Indexer**: The Node.js backend listens to Soroban contract events (`delivery_created`, `escrow_funded`, etc.) and indexes them into MongoDB for fast, queryable APIs.
- **Client**: The Next.js frontend integrates with Stellar wallets (e.g., Freighter) to allow users to sign transactions directly, while fetching historical and complex relational data from the backend API.

## Current Implementation Status
- **Completed Systems**: Escrow locking/releasing, Delivery state machine (Pending -> Delivered), Identity & Reputation (Tiering, KYC flags), Fleet Management (Invite/Accept/Treasury routing), Dispute Resolution (Admin split/refund/slash).
- **Partially Completed**: Cross-contract invocations are mostly wired, but `settlement_contract` is currently just a stub.
- **Missing/Unstable Areas**: The `settlement_contract` lacks actual AMM integration for cross-border asset swaps. TTL (Time-to-Live) management is functional but hardcoded (`518400`) and marked as WIP in some tests.

## Folder Structure Analysis
- `contracts/`: Contains all Soroban smart contracts as separate crates.
  - `shared_types/`: Central library for shared Enums, Structs, and Events. Prevents circular dependencies.
  - `escrow_contract/`: Core financial engine.
  - `delivery_contract/`: Delivery lifecycle manager.
  - `fleet_management_contract/`: SME fleet orchestration.
  - `identity_reputation_contract/`: Driver/User profiles and scoring.
  - `dispute_resolution_contract/`: Escrow freezing and arbitration.
  - `settlement_contract/`: Stubbed cross-border swap logic.
- `scripts/`: Bash scripts for deployment and initialization.
- `docs/`: Technical architecture and protocol documentation.

## Backend Architecture (Inferred)
- **Services/Event Listeners**: Background workers polling Soroban RPC for contract events.
- **Controllers**: REST APIs serving indexed data (e.g., user's delivery history, driver's reputation, fleet lists) to the frontend.
- **Repositories**: MongoDB adapters for querying indexed deliveries, user profiles, and fleet data.
- **Auth Flow**: Validates wallet signatures or session tokens generated after a wallet-sign challenge.

## Frontend Architecture (Inferred)
- **State Management**: Likely React Context or Zustand for managing wallet connection state and user roles (Sender, Driver, Fleet Owner).
- **API Integration**: React Query or SWR for fetching indexed data from the Node.js backend.
- **Blockchain Integration**: `@stellar/freighter-api` or similar to construct and sign Soroban transactions.

## Database Design (Inferred MongoDB Schema)
- **Deliveries**: `_id`, `deliveryId` (on-chain u64), `sender`, `recipient`, `driver`, `status`, `escrowAmount`, `metadata` (cargo info, origin, destination).
- **Users/Drivers**: `address`, `role`, `reputationScore`, `tier`, `deliveriesCompleted`, `kycVerified`, `fleetId`.
- **Fleets**: `fleetId`, `owner`, `treasury`, `totalActiveDrivers`.

## Authentication & Authorization
- **On-Chain**: Handled natively via Soroban's `caller.require_auth()`. The system heavily uses role-based checks (e.g., verifying `is_admin`, or ensuring the caller is the `sender`, `recipient`, or `driver`).
- **Off-Chain**: Standard JWT/session flow in the backend, initiated by cryptographic proof-of-wallet-ownership.

## Smart Contract / Blockchain Section
- **Architecture**: Modular. Contracts communicate via `env.invoke_contract()`. 
- **Escrow Flow**: Sender creates delivery -> Funds locked in Escrow -> Driver assigned -> Driver marks InTransit -> Recipient confirms -> Escrow triggers payout to driver (or fleet treasury).
- **Dispute Flow**: Either party raises dispute -> Escrow paused -> Admin resolves (refund, payout, or split) -> Driver reputation slashed if applicable.

## Environment Configuration
- `STELLAR_NETWORK`: Network to deploy to (testnet/mainnet).
- `SOROBAN_RPC_URL`: RPC endpoint for the Stellar network.
- `CONTRACT_DEPLOYER_KEY`: Secret key of the admin/deployer (DO NOT EXPOSE).
- `NETWORK_PASSPHRASE`: Stellar network passphrase.

## Development Conventions
- **Naming**: `snake_case` for functions/variables, `PascalCase` for Structs/Enums.
- **File Organization**: Contracts are organized as standard Rust workspaces. `lib.rs` contains the contract logic, `test.rs` contains unit tests.
- **Error Handling**: Custom `contracterror` enums (e.g., `SwiftChainError`, `EscrowError`) with explicit `u32` discriminants. Used with `panic_with_error!`.
- **Testing**: `env.mock_all_auths()` is heavily used in test files.
- **State Updates**: Pattern of `load -> mutate -> save -> extend_ttl`.

## Known Issues & Technical Debt
- **Settlement Stub**: `settlement_contract` contains a `todo!()` and is not fully implemented.
- **Hardcoded TTLs**: TTL values (`518400`) are hardcoded across multiple contracts instead of being managed dynamically or centrally configured in some instances.
- **Cross-Contract Complexity**: Tight coupling between Escrow, Delivery, and Dispute contracts requires careful deployment and initialization order.

## AI Agent Instructions
- **Preserve Modularity**: Do NOT merge separate contracts into a monolithic contract. Use cross-contract calls via `env.invoke_contract()`.
- **Use `shared_types`**: Any new struct or enum that needs to be passed between contracts MUST be defined in `shared_types`.
- **Always Require Auth**: Every state-mutating function must call `<address>.require_auth()` and validate the caller's role.
- **Event Emission**: Always emit an event for state changes. The off-chain backend relies entirely on these events.
- **Extend TTL**: Always extend the TTL of persistent storage keys after reading or writing to prevent archival.

## Recent Project Direction
- Recent focus has been on **Fleet Management** (Issues #67-72) and **Identity/Reputation** (Issue #73), allowing enterprise fleets to onboard drivers and route payouts to a central treasury.
- Dispute resolution logic was recently refined to include reputation slashing (`decrease_reputation`).

## Recommended Next Steps
1. Implement the AMM integration in `settlement_contract` to allow drivers to receive payouts in their preferred stablecoin/asset.
2. Refactor TTL extensions to use a centralized constant or configuration.
3. Build comprehensive integration tests that simulate the full lifecycle across all 7 contracts simultaneously.
