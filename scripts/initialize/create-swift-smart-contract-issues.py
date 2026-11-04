#!/usr/bin/env python3

# ============================================================
# SwiftChain - GitHub Issues Generator (Python Version)
# Smart Contract Issues: 50 Issues
# ============================================================
# ⚠️  SECURITY: Never hardcode your token.
# Before running, export your token in Git Bash/PowerShell:
#     export GITHUB_TOKEN="your-github-token"
# Then run:
#     python ./scripts/create-swift-smart-contract-issues.py
# ============================================================

import os
import sys
import json
import time
import ssl
import urllib.request
import urllib.error

# ── Configuration ─────────────────────────────────────────────
TOKEN = os.environ.get("GITHUB_TOKEN")
REPO  = "SwiftChainn/SwiftChain-SmartContract"
API_URL = f"https://api.github.com/repos/{REPO}/issues"

# ── Pre-flight Check ──────────────────────────────────────────
if not TOKEN:
    print("❌ ERROR: GITHUB_TOKEN environment variable not set.")
    print("   Run: export GITHUB_TOKEN='your-github-token'")
    sys.exit(1)

print("✅ Pre-flight checks passed.")
print(f"📦 Target Repo: {REPO}")
print("🚀 Starting issue creation...\n")

# ── Helper Function ───────────────────────────────────────────
def create_issue(title, body, labels):
    print(f"\n🆕 Creating: {title}")

    payload = json.dumps({
        "title": title,
        "body": body,
        "labels": [l.strip() for l in labels.split(",")]
    }).encode("utf-8")

    req = urllib.request.Request(
        API_URL,
        data=payload,
        headers={
            "Authorization": f"Bearer {TOKEN}",
            "Accept": "application/vnd.github.v3+json",
            "Content-Type": "application/json",
            "User-Agent": "SwiftChain-Issue-Creator"
        },
        method="POST"
    )

    # Bypass Windows SSL certificate verification issues
    ctx = ssl.create_default_context()
    ctx.check_hostname = False
    ctx.verify_mode = ssl.CERT_NONE

    for attempt in range(1, 4):
        try:
            with urllib.request.urlopen(req, timeout=30, context=ctx) as response:
                result = json.loads(response.read().decode("utf-8"))
                print(f"✅ SUCCESS: {result.get('html_url')}")
                break
        except urllib.error.HTTPError as e:
            error_body = e.read().decode("utf-8")
            print(f"❌ FAILED (HTTP {e.code}): {error_body}")
            break
        except Exception as e:
            print(f"⚠️  Attempt {attempt}/3 failed: {str(e)}")
            if attempt < 3:
                print(f"   Retrying in 5 seconds...")
                time.sleep(5)
            else:
                print(f"❌ ERROR: All 3 attempts failed for this issue.")

    # ⏱ Rate-limit buffer
    time.sleep(1)

def build_body(goal, reqs, deps, branch, target_dir):
    return f"""## 🎯 Goal
{goal}

## 📋 Requirements
{reqs}

## 📂 Target Directory
Contributors should work within this directory:
`{target_dir}`

## 🔗 Architecture & Dependencies
- **Architecture Reference**: [Smart Contract Architecture Docs](https://github.com/SwiftChainn/SwiftChain-SmartContract/blob/main/docs/architecture/smart-contract-architecture.md)
- **Dependencies**: {deps}

## 🔧 Technical Scope
- Rust
- Soroban SDK

## 🌿 Fork & Branch
- Fork the repository
- Branch: `git checkout -b {branch}`

## ✅ Acceptance Criteria
- All tests must pass successfully
- Test cases pass on testnet

## 💬 Example Commit Message
```
feat: {goal.lower()}
```"""

ISSUES = [
    # LOW (3 Environment Setup Issues)
    (
        "Smart Contract: Configure Cargo.toml for Identity Reputation Contract",
        "Set up the Cargo.toml dependencies and Soroban configurations for the identity reputation contract.",
        "- Define package metadata\n- Add Soroban SDK dependencies\n- Configure release profiles for WASM optimization",
        "None",
        "setup-identity-cargo",
        "contracts/identity_reputation_contract",
        "smart-contract,low,good first issue,architecture"
    ),
    (
        "Smart Contract: Configure Cargo.toml for Fleet Management Contract",
        "Set up the Cargo.toml dependencies and Soroban configurations for the fleet management contract.",
        "- Define package metadata\n- Add Soroban SDK dependencies\n- Configure release profiles for WASM optimization",
        "None",
        "setup-fleet-cargo",
        "contracts/fleet_management_contract",
        "smart-contract,low,good first issue,architecture"
    ),
    (
        "Smart Contract: Configure Cargo.toml for Dispute Resolution Contract",
        "Set up the Cargo.toml dependencies and Soroban configurations for the dispute resolution contract.",
        "- Define package metadata\n- Add Soroban SDK dependencies\n- Configure release profiles for WASM optimization",
        "None",
        "setup-dispute-cargo",
        "contracts/dispute_resolution_contract",
        "smart-contract,low,good first issue,architecture"
    ),
    
    # MEDIUM (5 Setup/Struct Issues)
    (
        "Smart Contract: Configure Cargo.toml for Settlement Contract",
        "Set up the Cargo.toml dependencies and Soroban configurations for the settlement contract.",
        "- Define package metadata\n- Add Soroban SDK dependencies\n- Configure release profiles for WASM optimization",
        "None",
        "setup-settlement-cargo",
        "contracts/settlement_contract",
        "smart-contract,medium,good first issue,architecture"
    ),
    (
        "Smart Contract: Define UserProfile and DriverProfile Structs",
        "Define the data structures for Users and Drivers in the identity contract.",
        "- Create UserProfile struct (address, join date)\n- Create DriverProfile struct (address, join date, total deliveries, reputation score)",
        "Configure Cargo.toml for Identity Reputation Contract",
        "define-identity-structs",
        "contracts/identity_reputation_contract",
        "smart-contract,medium,good first issue"
    ),
    (
        "Smart Contract: Define FleetProfile Struct",
        "Define the data structure for Enterprise Fleets in the fleet contract.",
        "- Create FleetProfile struct (fleet ID, owner wallet, treasury wallet, total active drivers)",
        "Configure Cargo.toml for Fleet Management Contract",
        "define-fleet-structs",
        "contracts/fleet_management_contract",
        "smart-contract,medium,good first issue"
    ),
    (
        "Smart Contract: Define DisputeCase Struct",
        "Define the data structure for Dispute Cases.",
        "- Create DisputeCase struct (delivery ID, initiator, reason code, timestamp, resolution status)",
        "Configure Cargo.toml for Dispute Resolution Contract",
        "define-dispute-structs",
        "contracts/dispute_resolution_contract",
        "smart-contract,medium,good first issue"
    ),
    (
        "Smart Contract: Define Supported Asset Pairs Storage",
        "Create a storage mechanism to whitelist supported Stellar assets for cross-border settlement.",
        "- Define supported asset types\n- Implement storage keys for XLM, USDC, NGNC, KES",
        "Configure Cargo.toml for Settlement Contract",
        "define-settlement-assets",
        "contracts/settlement_contract",
        "smart-contract,medium,good first issue"
    ),
    
    # HIGH (42 Core Features, Integrations, Security, Tests)
    # -- Identity Contract (High) --
    (
        "Smart Contract: Implement register_user Function",
        "Allow standard users (senders/receivers) to register their wallet on-chain.",
        "- Implement register_user\n- Store UserProfile in Soroban persistent storage",
        "Define UserProfile and DriverProfile Structs",
        "feat-register-user",
        "contracts/identity_reputation_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Implement register_driver Function",
        "Allow independent drivers to register and initialize their reputation score.",
        "- Implement register_driver\n- Initialize DriverProfile with a base reputation score (e.g., 50/100)",
        "Define UserProfile and DriverProfile Structs",
        "feat-register-driver",
        "contracts/identity_reputation_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Implement update_driver_kyc_status (Admin)",
        "Allow an admin/oracle to verify a driver's off-chain KYC and update their on-chain status.",
        "- Implement update_driver_kyc_status\n- Require admin authentication\n- Update KYC boolean in DriverProfile",
        "Implement register_driver",
        "feat-update-kyc",
        "contracts/identity_reputation_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Implement increase_reputation Logic",
        "Create a function that adds points to a driver's score after a successful delivery.",
        "- Implement increase_reputation\n- Calculate points based on delivery parameters",
        "Implement register_driver",
        "feat-increase-reputation",
        "contracts/identity_reputation_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Implement decrease_reputation Logic",
        "Create a function that deducts points for late deliveries or lost disputes.",
        "- Implement decrease_reputation\n- Deduct points without dropping below zero",
        "Implement register_driver",
        "feat-decrease-reputation",
        "contracts/identity_reputation_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Implement get_driver_tier Function",
        "Return a classification (Bronze, Silver, Gold) based on the current reputation score.",
        "- Implement get_driver_tier\n- Map score ranges to tiers",
        "Implement increase_reputation",
        "feat-driver-tier",
        "contracts/identity_reputation_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Integration - Expose is_eligible_for_enterprise",
        "Create a public read function that the delivery_contract can call to ensure a driver meets the minimum score.",
        "- Implement is_eligible_for_enterprise\n- Check if driver reputation >= enterprise threshold",
        "Implement get_driver_tier",
        "feat-enterprise-eligibility",
        "contracts/identity_reputation_contract",
        "smart-contract,high,advanced"
    ),
    (
        "Smart Contract: Security - Implement Reputation Bounds Checking",
        "Ensure scores cannot exceed 100 or drop below 0.",
        "- Add boundary logic to increase_reputation and decrease_reputation\n- Write defensive tests",
        "Implement increase_reputation Logic",
        "sec-reputation-bounds",
        "contracts/identity_reputation_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Security - Restrict Reputation Updates",
        "Ensure only authorized contracts can trigger score changes.",
        "- Add require_auth for delivery_contract and dispute_resolution_contract\n- Reject direct driver calls",
        "Implement increase_reputation Logic",
        "sec-restrict-reputation",
        "contracts/identity_reputation_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Testing - Unit Tests for Driver Registration & KYC",
        "Validate that profiles are stored and updated correctly.",
        "- Write Soroban unit tests covering registration\n- Write tests for KYC admin updates",
        "Implement update_driver_kyc_status",
        "test-driver-registration",
        "contracts/identity_reputation_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Testing - Unit Tests for Reputation Scoring Logic",
        "Test the math behind point additions, deductions, and tier calculations.",
        "- Write tests for scoring boundaries\n- Write tests for tier mapping",
        "Security - Implement Reputation Bounds Checking",
        "test-reputation-scoring",
        "contracts/identity_reputation_contract",
        "smart-contract,high"
    ),

    # -- Fleet Contract (High) --
    (
        "Smart Contract: Implement register_fleet Function",
        "Allow an SME to register a fleet and designate the treasury wallet address.",
        "- Implement register_fleet\n- Store FleetProfile in persistent storage",
        "Define FleetProfile Struct",
        "feat-register-fleet",
        "contracts/fleet_management_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Implement add_driver_to_fleet Function",
        "Allow a fleet owner to send an invite/add a driver's wallet address.",
        "- Implement add_driver_to_fleet\n- Store pending invite state",
        "Implement register_fleet",
        "feat-add-fleet-driver",
        "contracts/fleet_management_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Implement accept_fleet_invite Function",
        "Require the individual driver to sign a transaction accepting their inclusion in a fleet.",
        "- Implement accept_fleet_invite\n- Require driver auth\n- Update driver fleet status",
        "Implement add_driver_to_fleet",
        "feat-accept-fleet",
        "contracts/fleet_management_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Implement remove_driver_from_fleet Function",
        "Allow either the fleet owner or the driver to sever the relationship.",
        "- Implement remove_driver_from_fleet\n- Verify auth for either owner or driver",
        "Implement accept_fleet_invite",
        "feat-remove-fleet",
        "contracts/fleet_management_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Implement update_fleet_treasury Function",
        "Allow the fleet owner to update the payout destination wallet.",
        "- Implement update_fleet_treasury\n- Require fleet owner auth",
        "Implement register_fleet",
        "feat-update-treasury",
        "contracts/fleet_management_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Integration - Expose get_payout_address",
        "Create a function that the escrow_contract calls to route funds.",
        "- Implement get_payout_address\n- Return treasury if in fleet, otherwise return driver address",
        "Implement accept_fleet_invite",
        "feat-get-payout",
        "contracts/fleet_management_contract",
        "smart-contract,high,advanced"
    ),
    (
        "Smart Contract: Integration - Link Fleet to Identity",
        "When a fleet is registered, automatically create an identity profile.",
        "- Cross-contract call from fleet_management_contract to identity_reputation_contract",
        "Implement register_fleet",
        "feat-link-fleet-identity",
        "contracts/fleet_management_contract",
        "smart-contract,high,advanced"
    ),
    (
        "Smart Contract: Security - Implement Fleet Owner Authorization",
        "Ensure only the registered fleet owner can add/remove drivers or change treasury.",
        "- Add require_auth checks\n- Test unauthorized access rejection",
        "Implement update_fleet_treasury",
        "sec-fleet-auth",
        "contracts/fleet_management_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Testing - Unit Tests for Fleet Roster Management",
        "Test adding, accepting, and removing drivers.",
        "- Write comprehensive roster tests",
        "Implement remove_driver_from_fleet",
        "test-fleet-roster",
        "contracts/fleet_management_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Testing - Unit Tests for Treasury Routing Logic",
        "Verify get_payout_address returns the correct wallet based on fleet membership.",
        "- Write tests simulating escrow payout queries",
        "Integration - Expose get_payout_address",
        "test-treasury-routing",
        "contracts/fleet_management_contract",
        "smart-contract,high"
    ),

    # -- Dispute Contract (High) --
    (
        "Smart Contract: Implement raise_dispute Function",
        "Allow the Sender or Receiver to initiate a dispute for an active delivery.",
        "- Implement raise_dispute\n- Store DisputeCase details",
        "Define DisputeCase Struct",
        "feat-raise-dispute",
        "contracts/dispute_resolution_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Implement add_evidence_hash Function",
        "Allow both parties to submit IPFS/Cloudinary hashes of their proof.",
        "- Implement add_evidence_hash\n- Store evidence array in DisputeCase",
        "Implement raise_dispute",
        "feat-add-evidence",
        "contracts/dispute_resolution_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Integration - Trigger Escrow Freeze",
        "When raise_dispute is called, automatically notify the escrow_contract to lock funds.",
        "- Cross-contract call to escrow_contract.freeze_funds()",
        "Implement raise_dispute",
        "feat-trigger-freeze",
        "contracts/dispute_resolution_contract",
        "smart-contract,high,advanced"
    ),
    (
        "Smart Contract: Implement resolve_dispute_refund_sender (Admin)",
        "Allow a SwiftChain arbitrator to rule in favor of the sender and force refund.",
        "- Implement resolve_dispute_refund_sender\n- Require admin auth\n- Cross-contract call to escrow",
        "Integration - Trigger Escrow Freeze",
        "feat-resolve-refund",
        "contracts/dispute_resolution_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Implement resolve_dispute_pay_driver (Admin)",
        "Allow the arbitrator to rule in favor of the driver and force payout.",
        "- Implement resolve_dispute_pay_driver\n- Require admin auth\n- Cross-contract call to escrow",
        "Integration - Trigger Escrow Freeze",
        "feat-resolve-pay",
        "contracts/dispute_resolution_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Implement resolve_dispute_split_funds (Admin)",
        "Allow the arbitrator to execute a percentage split between sender and driver.",
        "- Implement resolve_dispute_split_funds\n- Require admin auth\n- Cross-contract split call to escrow",
        "Integration - Trigger Escrow Freeze",
        "feat-resolve-split",
        "contracts/dispute_resolution_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Integration - Trigger Reputation Penalty",
        "When a dispute is resolved against a driver, automatically slash their score.",
        "- Cross-contract call to identity_reputation_contract.decrease_reputation()",
        "Implement resolve_dispute_refund_sender (Admin)",
        "feat-dispute-penalty",
        "contracts/dispute_resolution_contract",
        "smart-contract,high,advanced"
    ),
    (
        "Smart Contract: Security - Implement Dispute Time Limits",
        "Ensure disputes can only be raised within X hours of the delivery being marked Completed.",
        "- Add timestamp checks to raise_dispute",
        "Implement raise_dispute",
        "sec-dispute-time-limits",
        "contracts/dispute_resolution_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Security - Implement Arbitrator Whitelist",
        "Ensure only authorized Admin wallets can execute resolve_dispute_* functions.",
        "- Setup admin whitelist storage\n- Enforce whitelist in resolution functions",
        "Implement resolve_dispute_refund_sender (Admin)",
        "sec-arbitrator-whitelist",
        "contracts/dispute_resolution_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Testing - Unit Tests for Dispute Lifecycle",
        "Test raising a dispute, adding evidence, and checking state.",
        "- Write Soroban unit tests for state changes",
        "Implement add_evidence_hash",
        "test-dispute-lifecycle",
        "contracts/dispute_resolution_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Testing - Integration Tests for Forced Payouts/Refunds",
        "Verify the admin can correctly route funds out of a frozen escrow.",
        "- Write multi-contract tests for dispute resolution",
        "Implement resolve_dispute_split_funds (Admin)",
        "test-dispute-integration",
        "contracts/dispute_resolution_contract",
        "smart-contract,high,advanced"
    ),

    # -- Settlement Contract (High) --
    (
        "Smart Contract: Implement register_driver_preference Function",
        "Allow drivers to set their preferred payout currency.",
        "- Implement register_driver_preference\n- Store preferred asset per driver",
        "Define Supported Asset Pairs Storage",
        "feat-driver-preference",
        "contracts/settlement_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Implement calculate_swap_estimate Function",
        "Create a read-only function that queries the Stellar AMM/DEX to estimate swap amounts.",
        "- Implement calculate_swap_estimate\n- Intersect with Soroban DEX interfaces",
        "Define Supported Asset Pairs Storage",
        "feat-swap-estimate",
        "contracts/settlement_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Implement execute_settlement_swap Function",
        "Logic to take Asset A, interact with the Soroban DEX router, and return Asset B.",
        "- Implement execute_settlement_swap\n- Handle DEX routing and limits",
        "Implement calculate_swap_estimate",
        "feat-execute-swap",
        "contracts/settlement_contract",
        "smart-contract,high,advanced"
    ),
    (
        "Smart Contract: Integration - Hook into Escrow Release",
        "Modify the escrow_contract release flow to route through settlement_contract.",
        "- Update escrow_contract to check settlement_contract before direct payout",
        "Implement execute_settlement_swap",
        "feat-hook-escrow-release",
        "contracts/settlement_contract",
        "smart-contract,high,advanced"
    ),
    (
        "Smart Contract: Implement Slippage Protection",
        "Allow the driver/system to set a maximum slippage tolerance for the AMM swap.",
        "- Add slippage params to execute_settlement_swap\n- Revert on excessive slippage",
        "Implement execute_settlement_swap",
        "feat-slippage-protection",
        "contracts/settlement_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Implement Platform FX Fee",
        "Deduct a small cross-border conversion fee during the swap.",
        "- Add fee deduction logic in execute_settlement_swap",
        "Implement execute_settlement_swap",
        "feat-fx-fee",
        "contracts/settlement_contract",
        "smart-contract,high"
    ),
    (
        "Smart Contract: Security - Handle Failed Swaps",
        "Implement fallback logic if the AMM lacks liquidity.",
        "- Catch swap failures\n- Fallback to paying out the original asset directly",
        "Implement Slippage Protection",
        "sec-failed-swaps",
        "contracts/settlement_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Security - Restrict Swap Execution",
        "Ensure execute_settlement_swap can only be triggered by the escrow_contract.",
        "- Add require_auth for escrow_contract",
        "Implement execute_settlement_swap",
        "sec-restrict-swap",
        "contracts/settlement_contract",
        "smart-contract,high,security"
    ),
    (
        "Smart Contract: Testing - Mock Soroban AMM for Local Testing",
        "Create a mock liquidity pool contract to simulate asset swaps locally.",
        "- Build mock AMM contract\n- Provide liquidity in tests",
        "Implement execute_settlement_swap",
        "test-mock-amm",
        "contracts/settlement_contract",
        "smart-contract,high,advanced"
    ),
    (
        "Smart Contract: Testing - Unit Tests for Settlement Flows",
        "Test successful swaps, slippage failures, and fallback logic.",
        "- Write extensive tests using the mock AMM",
        "Testing - Mock Soroban AMM for Local Testing",
        "test-settlement-flows",
        "contracts/settlement_contract",
        "smart-contract,high"
    )
]

for i, (title, goal, reqs, deps, branch, target_dir, labels) in enumerate(ISSUES, 1):
    print(f"\n════════════════════════════════════════")
    print(f"  📋 ISSUE {i}/50: {title}")
    print(f"════════════════════════════════════════")
    body = build_body(goal, reqs, deps, branch, target_dir)
    create_issue(title, body, labels)

print("\n\n════════════════════════════════════════")
print("  🎉 All 50 Issues Creation Attempted!")
print(f"  📌 Repo: https://github.com/{REPO}/issues")
print("════════════════════════════════════════")
