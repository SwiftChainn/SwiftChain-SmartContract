# SwiftChain Development Roadmap

## Current Milestone Progress
We are currently transitioning from **Phase 2 (Logistics Expansion)** into **Phase 3 (Full Blockchain Logistics Protocol)**. 
The core escrow engine, delivery state machine, driver reputation system, and enterprise fleet management are functional. The immediate focus is unlocking cross-border capabilities via the settlement contract.

## Completed Milestones
- [x] **Phase 1: Escrow MVP**
  - Payment locking and releasing logic.
  - Basic refund mechanisms.
- [x] **Phase 2: Logistics Expansion**
  - Delivery lifecycle tracking (Pending -> Active -> InTransit -> Delivered).
  - Dispute resolution protocol (Freeze, Split, Slash).
  - Identity & Reputation scoring (Bronze, Silver, Gold tiers).
  - Fleet Management (Enterprise treasury routing, driver invites).

## Pending Milestones
- [ ] **Phase 3: Cross-Border & Settlement**
  - Complete `settlement_contract` for AMM-based token swaps.
  - Allow drivers to select preferred payout assets (e.g., USDC vs local stablecoins).
- [ ] **Phase 4: Decentralized Governance & Oracles**
  - Replace centralized Admin dispute resolution with a decentralized Oracle/DAO mechanism.
  - Implement automated Proof-of-Delivery (PoD) verification.
- [ ] **Phase 5: Production Launch**
  - Comprehensive smart contract security audit.
  - Mainnet deployment and integration with production frontend/backend.

## Future Scaling Considerations
- **State Archival**: Soroban archives old state. The platform currently relies on hardcoded TTL extensions (`518400` ledgers). A robust strategy for restoring archived delivery/escrow states must be implemented off-chain if historical disputes arise.
- **Indexer Bottleneck**: As transaction volume grows, the Node.js backend indexer must be highly scalable to process Soroban events without lagging behind the ledger.
- **Cross-Contract Call Gas Limits**: Soroban has limits on cross-contract calls and execution resources. The heavy interaction between Delivery, Escrow, Dispute, and Identity contracts must be optimized to ensure transaction fees remain low.

## Recommended Implementation Order
1. **Settlement Logic**: Finish `settlement_contract` to unlock cross-border utility.
2. **E2E Testing**: Develop an end-to-end integration test suite covering the full lifecycle (Create -> Assign -> Transit -> Dispute -> Slash -> Resolve -> Swap -> Payout).
3. **TTL Refactor**: Centralize and configure TTL values to be dynamic or easily updatable.
4. **Testnet Beta**: Deploy all 7 contracts to Stellar Testnet and sync with the Next.js/Node.js stack.
5. **Security Audit**: Engage external auditors for the Soroban codebase.

## Production-Readiness Checklist
- [ ] Smart contracts audited by a reputable Web3 security firm.
- [ ] Admin keys secured via multisig (e.g., Stellar native multisig).
- [ ] Protocol platform fee dynamically configurable and tested.
- [ ] Off-chain indexer tested for high-throughput and network re-orgs.
- [ ] Fallback UI implemented for wallet connection failures.
- [ ] Mainnet `.env` configurations properly secured in CI/CD.
