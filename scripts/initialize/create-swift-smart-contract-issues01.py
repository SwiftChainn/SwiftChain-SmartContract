#!/usr/bin/env python3

# ============================================================
# SwiftChain - GitHub [Doc] Issues Generator (Python Version)
# Documentation Issues: 70 Issues
# Coverage: Phase 1 (Escrow/Foundation), Phase 2 (Logistics
#           Expansion), Phase 3 (Cross-Border & Settlement)
# ============================================================
# ⚠️  SECURITY: Never hardcode your token.
# Before running, export your token in Git Bash/PowerShell:
#     export GITHUB_TOKEN="your-github-token"
# Then run:
#     python ./scripts/create-swift-smart-contract-issues01.py
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

def build_body(description, goal, reqs, branch, target_dir):
    return f"""## 📁 Implementation Directory
Contributors should work within this directory (create `.md` documentation files here):
`{target_dir}`

## 🎯 Goal
{goal}

## #\tDescription
{description}

## 📋 Requirements
{reqs}

## 🔧 Technical Scope
- Documentation (Markdown)
- Rust / Soroban Smart Contract references
- Architecture Reference: [Smart Contract Architecture Docs](https://github.com/SwiftChainn/SwiftChain-SmartContract/blob/main/docs/architecture/smart-contract-architecture.md)

## 🌿 Branch
- Fork the repository
- Branch: `git checkout -b {branch}`

## ✅ Acceptance Criteria
- Documentation follows existing `docs/` conventions and structure
- All referenced functions, structs, and events exist in the contract source under `contracts/`
- Clear, contributor-friendly, and technically accurate content
- No hardcoded secrets, tokens, or sensitive values

## 📌 PR Requirements
- Open a Pull Request against the `main` branch
- Reference this issue in the PR description (e.g., `Closes #<issue-number>`)
- Keep the diff scoped to the documented contract/directory
- Update the relevant `docs/` file(s) under the Implementation Directory"""

ISSUES = [
    # ============================================================
    # PHASE 1 — ESCROW MVP & SHARED FOUNDATION (19 issues)
    # ============================================================

    # ── Shared Foundation (5) ──
    (
        "[Doc] Shared Core Data Types in shared_types",
        "Document the shared core data types (`DeliveryRecord`, `EscrowRecord`, `UserProfile`, `DriverProfile`, `FleetProfile`, `DisputeCase`), their field meanings, on-chain serialization, and how the shared crate prevents circular dependencies between the 7 contracts.",
        "Document every shared struct in `contracts/shared_types/lib.rs`.",
        "- Cover each core data type and every field\n- Explain on-chain serialization (Soroban `#[contracttype]`)\n- Explain why `shared_types` exists (avoids circular deps)\n- Reference the consuming contracts for each type",
        "doc-shared-core-types",
        "docs/shared_types/core-data-types"
    ),
    (
        "[Doc] Shared Error Types Across Contracts",
        "Document the shared error-type architecture: `SwiftChainError` and contract-specific `#[contracterror]` enums with explicit `u32` discriminants, plus the `panic_with_error!` convention used across all contracts.",
        "Document all error enums in `shared_types` and per-contract error definitions.",
        "- Enumerate every custom error variant across all contracts\n- Explain `u32` discriminants and decoding\n- Explain `panic_with_error!` usage\n- Note how error codes surface to backend/frontend",
        "doc-shared-errors",
        "docs/shared_types/error-types"
    ),
    (
        "[Doc] Protocol Event Catalog",
        "Write the protocol event catalog: every `env.events().publish(...)` event (topics + payload) across all contracts, and how the off-chain Node.js indexer consumes them to rebuild MongoDB read models.",
        "Document all on-chain events emitted by the 7 contracts.",
        "- List each event topic + payload per contract\n- Define the indexer subscription flow\n- Map events to MongoDB collections/read models",
        "doc-event-catalog",
        "docs/indexer/event-catalog"
    ),
    (
        "[Doc] Storage Key Namespace Definitions",
        "Document the shared storage-key namespace scheme (`DataKey` enums per contract) and how contract state is namespaced to avoid collisions.",
        "Document every `DataKey` variant and persistent storage namespace.",
        "- Enumerate `DataKey` enums per contract\n- Explain persistent vs temporary storage usage\n- Explain TTL extension on storage keys",
        "doc-storage-keys",
        "docs/shared_types/storage-keys"
    ),
    (
        "[Doc] Delivery Metadata and Cargo Descriptor Types",
        "Document the delivery metadata and cargo descriptor types (`DeliveryMetadata`, cargo info, origin/destination) and where they are stored on-chain vs off-chain.",
        "Document the delivery/cargo metadata types and their storage strategy.",
        "- Cover metadata struct fields\n- Distinguish on-chain vs off-chain storage\n- Explain metadata size and gas trade-offs",
        "doc-delivery-metadata",
        "docs/delivery_contract/metadata"
    ),

    # ── Escrow Setup & Config (2) ──
    (
        "[Doc] Escrow Contract Initialization and Query Functions",
        "Document `escrow_contract::init` (admin, token, platform fee args) and the protocol-config storage layout, plus the query functions `get_status`, `get_admin`, `get_token`, `get_protocol_version`.",
        "Document escrow initialization and all read-only query functions.",
        "- Cover `init` parameters and storage layout\n- Document each getter and its return type\n- Show example initialization flow",
        "doc-escrow-init",
        "docs/escrow_contract/initialization"
    ),
    (
        "[Doc] Platform Fee Configuration and Admin Update",
        "Document platform fee handling: `update_platform_fee`, `get_platform_fee`, basis-point semantics, and the admin-only guardrail.",
        "Document platform fee configuration for the escrow contract.",
        "- Explain basis-point (bps) fee semantics\n- Document admin-only authorization\n- Show fee update + validation flow",
        "doc-platform-fee",
        "docs/escrow_contract/platform-fee"
    ),

    # ── Escrow Lifecycle (4) ──
    (
        "[Doc] Escrow Creation and Payment Locking",
        "Write the escrow creation & payment-locking runbook (`create_escrow`): auth, token transfer to the contract, record storage, and emitted events.",
        "Document the escrow locking flow end-to-end.",
        "- Cover `create_escrow` signature and auth\n- Document token transfer to contract\n- Document record storage + emitted events\n- Note interplay with delivery creation",
        "doc-escrow-create",
        "docs/escrow_contract/create-lock"
    ),
    (
        "[Doc] Escrow Release on Delivery Confirmation",
        "Document `release_escrow`: the payout path, driver-vs-fleet-treasury routing, settlement hook, and platform fee deduction.",
        "Document the escrow payout/release flow.",
        "- Cover `release_escrow` caller requirements\n- Document driver vs fleet-treasury routing\n- Document settlement hook and platform fee deduction",
        "doc-escrow-release",
        "docs/escrow_contract/release-payout"
    ),
    (
        "[Doc] Escrow Refund Mechanism",
        "Document `refund_escrow` for failed deliveries/cancellations: caller rules, conditions, token flow back to sender, and reentrancy safeguards.",
        "Document the escrow refund flow.",
        "- Define refund eligibility conditions\n- Document token flow back to sender\n- Explain reentrancy safeguards",
        "doc-escrow-refund",
        "docs/escrow_contract/refund"
    ),
    (
        "[Doc] Escrow Balance Verification Guard",
        "Document the escrow balance-verification guard: how balances are asserted before state mutation, and the invariant that state and balance stay consistent.",
        "Document the balance guard invariant and enforcement.",
        "- Explain the balance-check guard mechanism\n- Document the state/balance consistency invariant\n- Note failure behavior when balance is wrong",
        "doc-escrow-balance-guard",
        "docs/escrow_contract/balance-guard"
    ),

    # ── Escrow Dispute (2) ──
    (
        "[Doc] Escrow Dispute Trigger and Freeze",
        "Document `raise_dispute` in escrow: freezing escrowed funds and the resulting pause on release/refund.",
        "Document the escrow freeze flow on dispute.",
        "- Cover `raise_dispute` auth and conditions\n- Document funds freezing behavior\n- Explain pause on release/refund while frozen",
        "doc-escrow-dispute-freeze",
        "docs/escrow_contract/dispute-freeze"
    ),
    (
        "[Doc] Admin Dispute Resolution and Escrow Settlement",
        "Document `resolve_dispute` (single resolution) and `resolve_dispute_split` (proportional split) in escrow: admin behavior, distribution, and finalization.",
        "Document admin dispute resolution in the escrow contract.",
        "- Cover `resolve_dispute` (pay driver variant)\n- Cover `resolve_dispute_split` (proportional split)\n- Document admin authorization and finalization",
        "doc-escrow-dispute-resolution",
        "docs/escrow_contract/dispute-resolution"
    ),

    # ── Escrow Admin/Ops (2) ──
    (
        "[Doc] Two-Step Admin Transfer Mechanism",
        "Document the two-step admin transfer (`propose_admin` / `accept_admin`) and the security rationale over single-step ownership changes.",
        "Document the admin ownership transfer flow.",
        "- Cover `propose_admin` / `accept_admin`\n- Explain the two-step rationale\n- Document security benefits and edge cases",
        "doc-admin-transfer",
        "docs/escrow_contract/admin-transfer"
    ),
    (
        "[Doc] Escrow TTL and Storage Ledger Extension",
        "Document escrow TTL / storage-ledger extension: which keys are extended, thresholds, and why active escrows stay alive against Soroban archival.",
        "Document the TTL/storage-extension design for escrow.",
        "- Enumerate keys requiring TTL extension\n- Document thresholds and extension constants\n- Explain archival risk and mitigations",
        "doc-escrow-ttl",
        "docs/escrow_contract/ttl-storage"
    ),

    # ── Escrow Events/Query/Testing (4) ──
    (
        "[Doc] Escrow Event Emission for All State Transitions",
        "Document the escrow event-emission requirements: the mandatory event for every state transition and the event shape consumed by the indexer.",
        "Document escrow event emission requirements.",
        "- List every emitted event on each transition\n- Document topic/payload shape\n- Explain indexer consumption",
        "doc-escrow-events",
        "docs/escrow_contract/events"
    ),
    (
        "[Doc] Get Escrow State Query Function",
        "Document the `get_escrow` query function and its off-chain read path (indexed escrow state in MongoDB).",
        "Document the escrow read/query function.",
        "- Cover `get_escrow` signature and return type\n- Document fields returned\n- Explain off-chain read path via indexer",
        "doc-escrow-query",
        "docs/escrow_contract/query"
    ),
    (
        "[Doc] Escrow Settlement Contract Hook",
        "Document the escrow settlement-contract hook (`set_settlement_contract` / `get_settlement_contract`) and when swap routing triggers on release.",
        "Document the escrow-to-settlement integration hook.",
        "- Cover the setter/getter functions\n- Explain when swap routing triggers\n- Document admin authorization for the setter",
        "doc-escrow-settlement-hook",
        "docs/escrow_contract/settlement-hook"
    ),
    (
        "[Doc] Escrow Integration Test Reference (Lifecycle)",
        "Create the escrow testing reference: the 16 integration tests, `mock_all_auths()` usage, and the scenario coverage matrix (lock → release → refund → split).",
        "Document the escrow test suite and coverage matrix.",
        "- Enumerate the 16 tests and scenarios\n- Explain `mock_all_auths()` usage\n- Provide a coverage matrix (lock/release/refund/split)",
        "doc-escrow-tests",
        "docs/escrow_contract/tests"
    ),

    # ============================================================
    # PHASE 2 — LOGISTICS EXPANSION (37 issues)
    # ============================================================

    # ── Delivery Contract (11) ──
    (
        "[Doc] Delivery Contract Initialization and Escrow Linkage",
        "Document `delivery_contract::init` and the escrow-contract linkage (address storage + cross-contract call authorization).",
        "Document delivery contract initialization and escrow integration.",
        "- Cover `init` parameters and address storage\n- Document escrow contract linkage\n- Explain cross-contract call authorization",
        "doc-delivery-init",
        "docs/delivery_contract/initialization"
    ),
    (
        "[Doc] Delivery Order Creation with Auto ID Generation",
        "Document delivery creation with auto-incrementing `DeliveryId`, cargo/metadata handling, and create/escrow ordering assumptions.",
        "Document the delivery creation flow.",
        "- Cover `create_delivery` and auto-incrementing IDs\n- Document cargo/metadata handling\n- Explain creation/escrow ordering",
        "doc-delivery-create",
        "docs/delivery_contract/create-delivery"
    ),
    (
        "[Doc] Delivery State Machine Enforcement Layer",
        "Write the delivery state-machine spec (`validate_transition`): Pending → Active → InTransit → Delivered, plus cancel/dispute, with an allowed-transitions table.",
        "Document the delivery state machine rules.",
        "- Document `DeliveryStatus` enum\n- Provide allowed-transitions table\n- Explain `validate_transition` enforcement and errors",
        "doc-delivery-state-machine",
        "docs/delivery_contract/state-machine"
    ),
    (
        "[Doc] Driver Assignment to Delivery Order",
        "Document `assign_driver`: caller requirements, driver address assignment, and state constraint checks.",
        "Document the driver-assignment flow.",
        "- Cover `assign_driver` signature and auth\n- Document state constraint checks\n- Note emitted events",
        "doc-delivery-assign",
        "docs/delivery_contract/assign-driver"
    ),
    (
        "[Doc] Mark In-Transit Status Update",
        "Document `mark_in_transit`: driver-only auth, state guard, and event emission.",
        "Document the in-transit status transition.",
        "- Cover `mark_in_transit` auth requirements\n- Document state pre-condition\n- Note emitted events",
        "doc-delivery-in-transit",
        "docs/delivery_contract/in-transit"
    ),
    (
        "[Doc] Recipient Delivery Confirmation with Escrow Release",
        "Document `confirm_delivery`: recipient auth and the cross-contract `release_escrow` call on the escrow contract.",
        "Document the delivery-confirmation cross-contract flow.",
        "- Cover `confirm_delivery` recipient auth\n- Document the `release_escrow` cross-contract call\n- Explain delivery state finalization",
        "doc-delivery-confirm",
        "docs/delivery_contract/confirm-delivery"
    ),
    (
        "[Doc] Delivery Cancellation and Escrow Refund Trigger",
        "Document `cancel_delivery`: who may cancel, state rules, and the cross-contract `refund_escrow` trigger.",
        "Document the delivery-cancellation flow.",
        "- Cover `cancel_delivery` authorized callers\n- Document state rules for cancellation\n- Explain the `refund_escrow` cross-contract trigger",
        "doc-delivery-cancel",
        "docs/delivery_contract/cancel-delivery"
    ),
    (
        "[Doc] Delivery Dispute Trigger with Escrow Freeze",
        "Document the delivery `raise_dispute` path and its cross-contract escrow freeze behavior.",
        "Document the delivery dispute-trigger flow.",
        "- Cover `raise_dispute` in delivery\n- Document the cross-contract escrow freeze\n- Note state transitions on dispute",
        "doc-delivery-dispute",
        "docs/delivery_contract/dispute-trigger"
    ),
    (
        "[Doc] Delivery Contract Storage TTL Management",
        "Document the delivery TTL management: keys extended per read/write and archival risk for long-lived deliveries.",
        "Document delivery storage TTL and archival considerations.",
        "- Enumerate keys requiring TTL extension\n- Document per-read/write extension pattern\n- Explain archival risk for long deliveries",
        "doc-delivery-ttl",
        "docs/delivery_contract/ttl-storage"
    ),
    (
        "[Doc] Delivery Query Functions and Read Model",
        "Document the query functions (`get_delivery`, `get_driver_profile`) and off-chain read-model implications.",
        "Document delivery query functions and off-chain reads.",
        "- Cover `get_delivery` and `get_driver_profile`\n- Document return types and fields\n- Explain off-chain read-model implications",
        "doc-delivery-query",
        "docs/delivery_contract/query"
    ),
    (
        "[Doc] Delivery Integration Test Reference (Full Lifecycle)",
        "Create the delivery testing reference: the 13 integration tests and full-lifecycle coverage (create → assign → transit → confirm → escrow release).",
        "Document the delivery test suite and lifecycle coverage.",
        "- Enumerate the 13 tests and scenarios\n- Map each test to a lifecycle stage\n- Provide a lifecycle coverage matrix",
        "doc-delivery-tests",
        "docs/delivery_contract/tests"
    ),

    # ── Identity & Reputation (8) ──
    (
        "[Doc] UserProfile and DriverProfile Structs",
        "Document `UserProfile`/`DriverProfile` struct fields and the identity-contract storage model.",
        "Document the identity contract data structures.",
        "- Cover every field of both structs\n- Document storage model and keys\n- Explain default values on creation",
        "doc-identity-structs",
        "docs/identity_reputation_contract/structs"
    ),
    (
        "[Doc] register_user Function",
        "Document `register_user`: auth, defaults (reputation, tier), and idempotency/overwrite rules.",
        "Document the user-registration flow.",
        "- Cover `register_user` auth and signature\n- Document default profile values\n- Explain idempotency/overwrite behavior",
        "doc-register-user",
        "docs/identity_reputation_contract/register-user"
    ),
    (
        "[Doc] register_driver Function",
        "Document `register_driver`: separate from user registration, driver defaults, and eligibility flags.",
        "Document the driver-registration flow.",
        "- Cover `register_driver` auth and signature\n- Document driver defaults and base reputation\n- Explain eligibility flags",
        "doc-register-driver",
        "docs/identity_reputation_contract/register-driver"
    ),
    (
        "[Doc] update_driver_kyc_status (Admin)",
        "Document `update_driver_kyc_status`: admin-only, KYC flag semantics, and downstream tier/eligibility implications.",
        "Document the admin KYC-management flow.",
        "- Cover admin auth requirement\n- Document KYC boolean semantics\n- Explain downstream tier/eligibility impact",
        "doc-update-kyc",
        "docs/identity_reputation_contract/update-kyc"
    ),
    (
        "[Doc] Reputation Increase and Decrease Logic",
        "Document `increase_reputation` / `decrease_reputation`: callers, delta rules, bounds checking, and floor/ceiling behavior.",
        "Document the reputation mutation functions.",
        "- Cover both functions' callers and auth\n- Document delta-calculation rules\n- Explain floor/ceiling bounds (0..100)",
        "doc-reputation-logic",
        "docs/identity_reputation_contract/reputation"
    ),
    (
        "[Doc] Driver Tier System (Bronze/Silver/Gold)",
        "Document the `DriverTier` system (Bronze/Silver/Gold) and `get_driver_tier` thresholds.",
        "Document the driver tiering system.",
        "- Cover `DriverTier` enum values\n- Document score-range thresholds\n- Explain `get_driver_tier` mapping logic",
        "doc-driver-tier",
        "docs/identity_reputation_contract/driver-tier"
    ),
    (
        "[Doc] is_eligible_for_enterprise Integration",
        "Document `is_eligible_for_enterprise`: the tier/KYC combination rules and fleet-integration contract.",
        "Document enterprise eligibility logic.",
        "- Cover tier/KYC combination thresholds\n- Document fleet-integration usage\n- Explain the read contract with delivery/fleet",
        "doc-enterprise-eligibility",
        "docs/identity_reputation_contract/enterprise-eligibility"
    ),
    (
        "[Doc] Identity Security and Authorized Contract Restriction",
        "Document identity security: reputation bounds checking and the `is_authorized_contract` restriction mechanism for reputation updates.",
        "Document identity contract security controls.",
        "- Cover reputation bounds enforcement\n- Document `is_authorized_contract` restriction\n- Explain which contracts can mutate reputation",
        "doc-identity-security",
        "docs/identity_reputation_contract/security"
    ),

    # ── Fleet Management (6) ──
    (
        "[Doc] register_fleet Function and FleetProfile",
        "Document `register_fleet` and `FleetProfile`: owner, treasury, and fleet-id generation.",
        "Document the fleet-registration flow.",
        "- Cover `register_fleet` auth and signature\n- Document `FleetProfile` fields\n- Explain fleet-id generation",
        "doc-register-fleet",
        "docs/fleet_management_contract/register-fleet"
    ),
    (
        "[Doc] Fleet Roster Workflow (Add/Accept/Remove)",
        "Document the roster workflow: `add_driver_to_fleet`, `accept_fleet_invite`, `remove_driver_from_fleet`, and their auth + state rules (invite → active).",
        "Document the fleet roster management flow.",
        "- Cover each roster function and its auth\n- Document the invite → active state transition\n- Explain ownership/driver removal rights",
        "doc-fleet-roster",
        "docs/fleet_management_contract/roster"
    ),
    (
        "[Doc] update_fleet_treasury Function",
        "Document `update_fleet_treasury` and treasury routing semantics.",
        "Document treasury-update logic.",
        "- Cover `update_fleet_treasury` owner auth\n- Document new-treasury validation\n- Explain treasury as payout destination",
        "doc-update-treasury",
        "docs/fleet_management_contract/update-treasury"
    ),
    (
        "[Doc] get_payout_address Payout Routing",
        "Document `get_payout_address`: how escrow uses it to route driver payouts to a fleet treasury.",
        "Document the payout-address routing integration.",
        "- Cover `get_payout_address` behavior (fleet vs solo)\n- Document escrow usage\n- Explain return semantics",
        "doc-payout-address",
        "docs/fleet_management_contract/payout-address"
    ),
    (
        "[Doc] Fleet to Identity Integration Link",
        "Document the fleet↔identity linkage (`set_identity_contract`, driver eligibility via reputation).",
        "Document fleet-identity cross-contract integration.",
        "- Cover `set_identity_contract` wiring\n- Explain eligibility checks via identity\n- Document the cross-contract call path",
        "doc-fleet-identity-link",
        "docs/fleet_management_contract/identity-link"
    ),
    (
        "[Doc] Fleet Owner Authorization Security",
        "Document fleet security: owner authorization (`require_auth`) and the treasury as a canonical payout source.",
        "Document fleet security controls.",
        "- Cover owner `require_auth` enforcement\n- Document treasury canonical-source rule\n- Explain unauthorized-access rejection",
        "doc-fleet-security",
        "docs/fleet_management_contract/security"
    ),

    # ── Dispute Resolution (12) ──
    (
        "[Doc] raise_dispute Function",
        "Document `raise_dispute` in the dispute contract: caller eligibility and dispute creation rules.",
        "Document atomic dispute creation.",
        "- Cover `raise_dispute` authorized callers\n- Document `DisputeCase` creation defaults\n- Explain reason-code handling",
        "doc-raise-dispute",
        "docs/dispute_resolution_contract/raise-dispute"
    ),
    (
        "[Doc] add_evidence_hash Function",
        "Document `add_evidence_hash`: evidence storage model and hash format.",
        "Document the evidence-submission flow.",
        "- Cover `add_evidence_hash` auth\n- Document evidence array storage\n- Explain hash format and validation",
        "doc-add-evidence",
        "docs/dispute_resolution_contract/evidence"
    ),
    (
        "[Doc] Trigger Escrow Freeze Integration",
        "Document the escrow-freeze integration on dispute raise (cross-contract call path and ordering).",
        "Document the dispute→escrow freeze integration.",
        "- Cover the cross-contract `freeze_funds` call\n- Document call ordering and atomicity\n- Explain failure behavior if freeze fails",
        "doc-dispute-freeze-integration",
        "docs/dispute_resolution_contract/escrow-freeze"
    ),
    (
        "[Doc] resolve_dispute_refund_sender (Admin)",
        "Document `resolve_dispute_refund_sender`: admin action and final outcome.",
        "Document the sender-refund resolution path.",
        "- Cover admin auth requirement\n- Document the refund outcome on escrow\n- Explain dispute finalization",
        "doc-resolve-refund-sender",
        "docs/dispute_resolution_contract/resolve-refund-sender"
    ),
    (
        "[Doc] resolve_dispute_pay_driver (Admin)",
        "Document `resolve_dispute_pay_driver`: admin action, payout routing, and finalization.",
        "Document the pay-driver resolution path.",
        "- Cover admin auth requirement\n- Document payout routing\n- Explain dispute finalization",
        "doc-resolve-pay-driver",
        "docs/dispute_resolution_contract/resolve-pay-driver"
    ),
    (
        "[Doc] resolve_dispute_split_funds (Admin)",
        "Document `resolve_dispute_split_funds`: proportional split logic and admin control.",
        "Document the split resolution path.",
        "- Cover admin auth requirement\n- Document proportional split mechanics\n- Explain escrow cross-call for split payout",
        "doc-resolve-split-funds",
        "docs/dispute_resolution_contract/resolve-split-funds"
    ),
    (
        "[Doc] Arbitrator Whitelist Security",
        "Document the arbitrator whitelist (`add_admin`, `remove_admin`, `is_admin`) and admin-only resolution security.",
        "Document the arbitrator whitelist controls.",
        "- Cover `add_admin`/`remove_admin` auth\n- Document `is_admin` enforcement\n- Explain admin-only resolution guarantees",
        "doc-arbitrator-whitelist",
        "docs/dispute_resolution_contract/arbitrator-whitelist"
    ),
    (
        "[Doc] Dispute Time Limits Security",
        "Document dispute time limits: `get_dispute_time_limit` and enforcement on resolution.",
        "Document dispute time-limit enforcement.",
        "- Cover time-limit constants and storage\n- Document `get_dispute_time_limit`\n- Explain enforcement on raise/resolution",
        "doc-dispute-time-limits",
        "docs/dispute_resolution_contract/time-limits"
    ),
    (
        "[Doc] Reputation Penalty Integration",
        "Document the reputation-penalty integration: how driver-fault resolution triggers `decrease_reputation` on the identity contract.",
        "Document the dispute→reputation penalty integration.",
        "- Cover the cross-contract `decrease_reputation` call\n- Document when penalties trigger\n- Explain failure handling",
        "doc-reputation-penalty",
        "docs/dispute_resolution_contract/reputation-penalty"
    ),
    (
        "[Doc] get_dispute Query and DisputeCase Fields",
        "Document the `get_dispute` query function and `DisputeCase`/`reason_code` field semantics.",
        "Document the dispute query function and struct fields.",
        "- Cover `get_dispute` signature and return type\n- Document `DisputeCase` + `reason_code` fields\n- Explain off-chain read usage",
        "doc-get-dispute",
        "docs/dispute_resolution_contract/get-dispute"
    ),
    (
        "[Doc] Dispute Lifecycle Unit Test Reference",
        "Create the dispute lifecycle testing reference: the 14 unit tests and evidence/freeze/split scenarios.",
        "Document the dispute unit test suite.",
        "- Enumerate the 14 tests and scenarios\n- Cover evidence, freeze, split cases\n- Provide a lifecycle coverage matrix",
        "doc-dispute-tests",
        "docs/dispute_resolution_contract/tests"
    ),
    (
        "[Doc] Forced Payout/Refund Integration Test Reference",
        "Create the forced payout/refund integration-test reference: cross-contract tests between dispute, escrow, and identity.",
        "Document the cross-contract dispute test scenarios.",
        "- Enumerate cross-contract integration tests\n- Cover refund/payout/split routing\n- Document multi-contract env setup",
        "doc-dispute-integration-tests",
        "docs/dispute_resolution_contract/integration-tests"
    ),

    # ============================================================
    # PHASE 3 — CROSS-BORDER & SETTLEMENT (14 issues)
    # ============================================================

    # ── Setup/Config (3) ──
    (
        "[Doc] Settlement Contract Overview and Initialization",
        "Document the settlement contract overview & init: admin, platform FX fee, router, and escrow-contract wiring.",
        "Document settlement contract setup and configuration.",
        "- Cover `init` parameters and storage\n- Document router + escrow wiring\n- Explain platform FX fee storage",
        "doc-settlement-init",
        "docs/settlement_contract/initialization"
    ),
    (
        "[Doc] Supported Asset Pairs Storage",
        "Document supported asset-pair storage: `set_supported_asset`, `set_supported_asset_pair`, `is_supported_asset_pair`, `is_supported`.",
        "Document the supported-asset whitelist mechanism.",
        "- Cover setter functions and auth\n- Document `SupportedAsset` types (XLM, USDC, NGNC, KES)\n- Explain `is_supported` validation",
        "doc-supported-assets",
        "docs/settlement_contract/supported-assets"
    ),
    (
        "[Doc] Router and Escrow Contract Wiring",
        "Document the router configuration (`set_router`, `get_router`) and the mock/prod DEX address contract.",
        "Document DEX router configuration and the escrow wiring.",
        "- Cover `set_router`/`get_router` auth\n- Document mock vs prod router addresses\n- Explain escrow-contract linkage",
        "doc-router-wiring",
        "docs/settlement_contract/router-wiring"
    ),

    # ── Driver Preference (2) ──
    (
        "[Doc] register_driver_preference Function",
        "Document `register_driver_preference` / `get_driver_preference`: driver-selected payout asset and validation against supported pairs.",
        "Document the driver payout-preference flow.",
        "- Cover `register_driver_preference` auth\n- Document asset validation against supported pairs\n- Explain `get_driver_preference` returns",
        "doc-driver-preference",
        "docs/settlement_contract/driver-preference"
    ),
    (
        "[Doc] Preferred Asset Validation and Overwrite Rules",
        "Document the supported-pair validation and safe-overwrite rules for driver preferences.",
        "Document preference validation and determinism.",
        "- Cover supported-pair acceptance\n- Document safe, deterministic overwrites\n- Explain rejection on unsupported assets",
        "doc-preference-validation",
        "docs/settlement_contract/preference-validation"
    ),

    # ── Swap Engine (3) ──
    (
        "[Doc] calculate_swap_estimate Function",
        "Document `calculate_swap_estimate`: quote calculation, inputs/outputs, and use by off-chain UI.",
        "Document the swap-estimate read function.",
        "- Cover inputs (in, in asset, out asset)\n- Document returned estimate semantics\n- Explain off-chain UI usage",
        "doc-swap-estimate",
        "docs/settlement_contract/swap-estimate"
    ),
    (
        "[Doc] execute_settlement_swap Function",
        "Document `execute_settlement_swap`: the full swap execution path via the router.",
        "Document the swap-execution flow.",
        "- Cover auth and caller requirements\n- Document router interaction path\n- Explain returned output amounts",
        "doc-execute-swap",
        "docs/settlement_contract/execute-swap"
    ),
    (
        "[Doc] Slippage Protection",
        "Document slippage protection: how worst-case output is enforced to prevent adverse price movement.",
        "Document slippage-tolerance safeguards.",
        "- Cover slippage tolerance params\n- Document worst-case output enforcement\n- Explain revert-on-excessive-slippage",
        "doc-slippage",
        "docs/settlement_contract/slippage"
    ),

    # ── Security/Integration (3) ──
    (
        "[Doc] Platform FX Fee",
        "Document the platform FX fee: bps, application in swap, and admin update controls.",
        "Document the cross-border FX fee.",
        "- Cover fee-bps storage and semantics\n- Document fee deduction in swap\n- Explain admin update controls",
        "doc-fx-fee",
        "docs/settlement_contract/fx-fee"
    ),
    (
        "[Doc] Failed Swap Handling",
        "Document failed-swap handling: `set_fail_swap`, recovery path, and guarantees when a swap fails.",
        "Document swap-failure fallback logic.",
        "- Cover `set_fail_swap` test control\n- Document fallback to original asset payout\n- Explain liquidity-failure guarantees",
        "doc-failed-swaps",
        "docs/settlement_contract/failed-swaps"
    ),
    (
        "[Doc] Escrow-Only Swap Authorization and Routing Hook",
        "Document the escrow-only swap authorization and the escrow-routing integration hook (when escrow release invokes settlement for a driver's preferred asset).",
        "Document the settlement security boundary and escrow integration.",
        "- Cover escrow-only `require_auth` gate\n- Document the routing hook on escrow release\n- Explain security rationale for restricted execution",
        "doc-escrow-auth-hook",
        "docs/settlement_contract/escrow-auth-hook"
    ),

    # ── Testing (3) ──
    (
        "[Doc] Mock Soroban AMM for Local Testing",
        "Document the mock Soroban AMM (`quote_exact_input`, `set_rate`, `swap_exact_tokens_for_tokens`) and its role in local testing.",
        "Document the mock AMM contract used in tests.",
        "- Cover mock AMM functions and state\n- Document `set_rate`/`set_fail_swap` controls\n- Explain role in local swap testing",
        "doc-mock-amm",
        "docs/settlement_contract/mock-amm"
    ),
    (
        "[Doc] Settlement Unit Test Reference",
        "Create the settlement testing reference: the 15 unit tests and swap/preference/slippage/FX scenarios.",
        "Document the settlement unit test suite.",
        "- Enumerate the 15 tests and scenarios\n- Cover preference, slippage, FX cases\n- Provide a coverage matrix",
        "doc-settlement-tests",
        "docs/settlement_contract/tests"
    ),
    (
        "[Doc] Settlement Security Controls Reference",
        "Write the settlement security doc: all `require_auth` gates, escrow-only authorization, supported-pair guards, and failure atomicity.",
        "Document settlement security controls comprehensively.",
        "- Enumerate every `require_auth` gate\n- Document escrow-only authorization\n- Cover supported-pair guards and failure atomicity",
        "doc-settlement-security",
        "docs/settlement_contract/security"
    )
]

for i, (title, description, goal, reqs, branch, target_dir) in enumerate(ISSUES, 1):
    print(f"\n════════════════════════════════════════")
    print(f"  📋 ISSUE {i}/{len(ISSUES)}: {title}")
    print(f"════════════════════════════════════════")
    body = build_body(description, goal, reqs, branch, target_dir)
    create_issue(title, body, "smart-contract,high,documentation")

print("\n\n════════════════════════════════════════")
print(f"  🎉 All {len(ISSUES)} [Doc] Issues Creation Attempted!")
print(f"  📌 Repo: https://github.com/{REPO}/issues")
print("════════════════════════════════════════")