#!/usr/bin/env python3

import os
import sys
import ssl
import json
import urllib.request
import urllib.error
import urllib.parse

# ── Configuration ─────────────────────────────────────────────────────────────

REPO_OWNER = "SwiftChainn"
REPO_NAME  = "SwiftChain-SmartContract"
API_URL    = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/labels"

try:
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
except Exception:
    pass

# ── Auth ──────────────────────────────────────────────────────────────────────

GITHUB_TOKEN = os.environ.get("GITHUB_TOKEN")
if not GITHUB_TOKEN:
    print("ERROR: Environment variable GITHUB_TOKEN is not set.")
    print("       Export it before running: export GITHUB_TOKEN=<your_token>")
    raise SystemExit(1)

# ── Label definitions ─────────────────────────────────────────────────────────

LABELS = [
    {
        "name":        "frontend",
        "color":       "bfd4f2",
        "description": "Frontend / UI / client-side implementation",
    },
    {
        "name":        "high",
        "color":       "d93f0b",
        "description": "High complexity",
    },
    {
        "name":        "medium",
        "color":       "fbca04",
        "description": "Medium complexity",
    },
    {
        "name":        "low",
        "color":       "0e8a16",
        "description": "Low complexity",
    },
    {
        "name":        "smart-contract",
        "color":       "1d76db",
        "description": "Smart contract implementation",
    },
    {
        "name":        "good first issue",
        "color":       "7057ff",
        "description": "Good for newcomers",
    },
    {
        "name":        "advanced",
        "color":       "b60205",
        "description": "Advanced integration",
    },
    {
        "name":        "architecture",
        "color":       "0052cc",
        "description": "Architectural design and setup",
    },
    {
        "name":        "security",
        "color":       "e11d21",
        "description": "Security modifiers and protections",
    },
    {
        "name":        "documentation",
        "color":       "0075ca",
        "description": "Documentation and technical writing",
    }
]

# ── SSL context (certificate verification disabled) ───────────────────────────

ssl_ctx = ssl.create_default_context()
ssl_ctx.check_hostname = False
ssl_ctx.verify_mode    = ssl.CERT_NONE

# ── Helper ────────────────────────────────────────────────────────────────────

def create_label(label: dict) -> None:
    """Send a POST request to create a single GitHub label."""
    payload = json.dumps(label).encode("utf-8")

    req = urllib.request.Request(
        url=API_URL,
        data=payload,
        method="POST",
        headers={
            "Authorization": f"Bearer {GITHUB_TOKEN}",
            "Accept":        "application/vnd.github+json",
            "Content-Type":  "application/json",
            "User-Agent":    "create_label.py",
        },
    )

    try:
        with urllib.request.urlopen(req, context=ssl_ctx) as resp:
            if resp.status == 201:
                print(
                    f"  [✓] Created  '{label['name']}'  "
                    f"(#{label['color']})"
                )
            else:
                print(
                    f"  [!] Unexpected status {resp.status} "
                    f"for label '{label['name']}'"
                )

    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        if exc.code == 422 and "already_exists" in body:
            update_label(label)
        else:
            print(
                f"  [✗] Failed to create '{label['name']}' "
                f"— HTTP {exc.code}: {body}"
            )

# ── Update Helper ─────────────────────────────────────────────────────────────

def update_label(label: dict) -> None:
    """PATCH an existing GitHub label to match the desired color/description."""
    label_name = label["name"]
    update_url = f"{API_URL}/{urllib.parse.quote(label_name)}"
    payload = json.dumps(
        {
            "new_name": label_name,
            "color": label.get("color"),
            "description": label.get("description"),
        }
    ).encode("utf-8")

    req = urllib.request.Request(
        url=update_url,
        data=payload,
        method="PATCH",
        headers={
            "Authorization": f"Bearer {GITHUB_TOKEN}",
            "Accept":        "application/vnd.github+json",
            "Content-Type":  "application/json",
            "User-Agent":    "create_labels.py",
        },
    )

    try:
        with urllib.request.urlopen(req, context=ssl_ctx) as resp:
            if resp.status == 200:
                print(
                    f"  [~] Updated  '{label_name}'  "
                    f"(#{label.get('color')})"
                )
            else:
                print(
                    f"  [!] Unexpected status {resp.status} "
                    f"updating label '{label_name}'"
                )
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        print(
            f"  [✗] Failed to update '{label_name}' "
            f"— HTTP {exc.code}: {body}"
        )

# ── Main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    print("=" * 60)
    print(f"  Target repo: {REPO_OWNER}/{REPO_NAME}")
    print(f"  Endpoint   : {API_URL}")
    print("=" * 60)

    for label in LABELS:
        create_label(label)

    print("=" * 60)
    print("  Done — all labels processed.")
    print("=" * 60)


if __name__ == "__main__":
    main()
