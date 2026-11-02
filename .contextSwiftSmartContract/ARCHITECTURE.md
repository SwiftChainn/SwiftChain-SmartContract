# SwiftChain Technical Architecture

## System Boundaries
SwiftChain is designed using a CQRS (Command Query Responsibility Segregation) inspired hybrid Web3 architecture:
1. **On-Chain Smart Contracts (Command & Source of Truth)**: Handles all state mutations (creating deliveries, locking funds, resolving disputes). Written in Rust for Stellar Soroban.
2. **Off-Chain Indexer & Backend (Query & Read Model)**: A Node.js/MongoDB backend that listens to Soroban events. It reconstructs the relational state of the system for fast, complex queries (e.g., "Get all deliveries for Fleet X in the last month").
3. **Client Frontend (UI)**: A Next.js application where users connect their Stellar wallets (e.g., Freighter). It reads data from the backend and sends signed transaction commands directly to the Soroban RPC.

## Module Relationships (Smart Contracts)
The on-chain architecture is highly modular, split into 7 distinct domains:
- **`shared_types`**: The foundational library containing structs (`DeliveryRecord`, `EscrowRecord`) and enums (`DeliveryStatus`) used by all other contracts.
- **`delivery_contract`**: The state machine. Manages the logistics metadata and lifecycle.
- **`escrow_contract`**: The financial hub. Holds all assets. Only releases funds when `delivery_contract` or `dispute_resolution_contract` authorizes it.
- **`identity_reputation_contract`**: The CRM. Tracks user registration, driver KYC, and reputation scores.
- **`fleet_management_contract`**: The enterprise layer. Allows grouping of drivers and routing payouts to a central treasury.
- **`dispute_resolution_contract`**: The arbitration layer. Can freeze the `escrow_contract` and slash reputation in `identity_reputation_contract`.
- **`settlement_contract`**: The exchange layer. Intercepts payouts from `escrow_contract` to swap assets via DEXs.

## Request Lifecycle Examples

### 1. Create Delivery Flow
1. **User** clicks "Create Delivery" on Frontend.
2. **Frontend** constructs a transaction calling `escrow_contract::create_escrow` and `delivery_contract::create_delivery`.
3. **User Wallet** signs the transaction.
4. **Soroban RPC** executes the transaction. 
   - `escrow_contract` transfers tokens from the user to itself and stores an `EscrowRecord`.
   - `delivery_contract` generates a `DeliveryId` and stores a `DeliveryRecord`.
   - Both contracts emit events (`escrow_funded`, `delivery_created`).
5. **Backend Indexer** catches events, creates documents in MongoDB.
6. **Frontend** polls/websockets backend and updates UI.

### 2. Confirm Delivery & Payout Flow
1. **Recipient** signs `confirm_delivery` on `delivery_contract`.
2. `delivery_contract` updates state to `Delivered`.
3. `delivery_contract` makes a cross-contract call (`invoke_contract`) to `escrow_contract::release_escrow`.
4. `escrow_contract` checks `fleet_management_contract` to see if the driver belongs to a fleet.
   - If yes: Route payout to fleet treasury.
   - If no: Route payout to driver.
5. `escrow_contract` calls `settlement_contract` to swap assets if the driver prefers a different currency.
6. `escrow_contract` transfers the final asset to the destination and a platform fee to the Admin.
7. `delivery_contract` calls `identity_reputation_contract` to increase the driver's reputation score.

## Data Flow
- **Writes**: `Frontend -> Wallet Signature -> Soroban RPC -> Smart Contract State -> Event Emission`.
- **Reads**: `Smart Contract Event -> Node.js Indexer -> MongoDB -> REST/GraphQL API -> Frontend`.

## Infrastructure Assumptions
- **RPC Availability**: The system heavily relies on a stable Soroban RPC node for both transaction submission and event indexing.
- **Wallet Adoption**: Assumes drivers and users have access to Stellar-compatible wallets and enough XLM for transaction fees.
- **Clock Sync**: Dispute time limits rely on `env.ledger().timestamp()`. Assumes ledger timestamps are reasonably accurate.

## Scaling Considerations
- **State Expiry**: Stellar Soroban implements state archival (TTL). If a delivery takes months, its persistent storage might expire. The contracts explicitly call `extend_ttl()` on every read/write to keep active deliveries alive.
- **Cross-Contract Resource Limits**: Soroban limits CPU instructions and memory per transaction. The deep call stack (`delivery -> escrow -> fleet -> settlement`) must be profiled to ensure it doesn't exceed gas limits, causing transaction failures.

## Design Patterns Used
- **Role-Based Access Control (RBAC)**: Enforced via `require_auth()` and `is_admin()` checks.
- **State Machine**: Strict validation in `delivery_contract` ensures transitions (e.g., cannot go from `Pending` directly to `Delivered`).
- **Event-Sourcing (Off-chain)**: The contracts are lean, only storing what is necessary for consensus. All historical and relational data is offloaded to events for the backend to piece together.
