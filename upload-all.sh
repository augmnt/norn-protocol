#!/bin/zsh
set -e

# ── Deploy & Upload All 11 Wallet Contracts ──────────────────────────────────
#
# Deploys each contract to the devnet, captures the new loom IDs, uploads WASM
# bytecode, and prints the env vars for Vercel / .env.local.
#
# Usage:
#   NORN_WALLET_PASSWORD=<pw> ./upload-all.sh
#   NORN_WALLET_PASSWORD=<pw> ./upload-all.sh --upload-only  (skip deploy, use existing IDs)
#
# Requirements:
#   - norn CLI installed and wallet configured
#   - All WASM files already built (cargo build --release in each example dir)
#   - NORN_WALLET_PASSWORD set (or enter interactively 22 times)

RPC="http://seed.norn.network:9741"
BASE="/Users/chrisijoyah/Code/norn-protocol/examples"
OUTPUT_FILE="/tmp/norn-loom-ids.env"

# Contract name → directory name → wasm file name mapping
# Format: "app_name:dir_name:wasm_name"
contracts=(
  "escrow:escrow:escrow"
  "treasury:multisig-treasury:multisig_treasury"
  "vesting:vesting:vesting"
  "launchpad:launchpad:launchpad"
  "splitter:splitter:splitter"
  "crowdfund:crowdfund:crowdfund"
  "governance:governance:governance"
  "staking:staking:staking"
  "swap:swap:swap"
  "airdrop:airdrop:airdrop"
  "timelock:timelock:timelock"
)

# Associative array to hold loom IDs
typeset -A loom_ids

echo ""
echo "╔══════════════════════════════════════════════════╗"
echo "║  Norn Contract Deployment — 11 Wallet Contracts  ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "  RPC: $RPC"
echo ""

# ── Step 1: Deploy all contracts ─────────────────────────────────────────────

if [[ "$1" != "--upload-only" ]]; then
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  Step 1: Deploying looms"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""

  for entry in "${contracts[@]}"; do
    app_name="${entry%%:*}"
    rest="${entry#*:}"
    dir_name="${rest%%:*}"

    echo "  Deploying $app_name..."
    output=$(norn wallet deploy-loom --name "$app_name" --yes --rpc-url "$RPC" 2>&1)

    # Parse loom ID from output (format: "Loom ID: <64-char hex>")
    loom_id=$(echo "$output" | grep -oE 'Loom ID: [0-9a-f]{64}' | head -1 | awk '{print $3}')

    if [[ -z "$loom_id" ]]; then
      echo "  ✗ Failed to deploy $app_name"
      echo "  Output: $output"
      echo ""
      continue
    fi

    loom_ids[$app_name]="$loom_id"
    echo "  ✓ $app_name → $loom_id"

    # Small delay to let the block propagate
    sleep 4
  done

  echo ""
else
  echo "  --upload-only mode: reading existing loom IDs from seed..."
  echo ""

  # Fetch existing looms from the chain
  result=$(curl -s "$RPC" -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"norn_listLooms","params":[100, 0],"id":1}')

  for entry in "${contracts[@]}"; do
    app_name="${entry%%:*}"
    loom_id=$(echo "$result" | python3 -c "
import sys, json
data = json.load(sys.stdin)
for loom in data.get('result', []):
    if loom.get('name') == '$app_name':
        print(loom['loom_id'])
        break
" 2>/dev/null || true)

    if [[ -n "$loom_id" ]]; then
      loom_ids[$app_name]="$loom_id"
      echo "  ✓ $app_name → $loom_id"
    else
      echo "  ✗ $app_name not found on chain"
    fi
  done
  echo ""
fi

# ── Step 2: Upload bytecode ──────────────────────────────────────────────────

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Step 2: Uploading bytecode"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

for entry in "${contracts[@]}"; do
  app_name="${entry%%:*}"
  rest="${entry#*:}"
  dir_name="${rest%%:*}"
  wasm_name="${rest#*:}"

  loom_id="${loom_ids[$app_name]}"
  if [[ -z "$loom_id" ]]; then
    echo "  ⊘ Skipping $app_name (no loom ID)"
    continue
  fi

  wasm="$BASE/$dir_name/target/wasm32-unknown-unknown/release/$wasm_name.wasm"
  if [[ ! -f "$wasm" ]]; then
    echo "  ✗ WASM not found: $wasm"
    continue
  fi

  echo "  Uploading $app_name..."
  norn wallet upload-bytecode --loom-id "$loom_id" --bytecode "$wasm" --rpc-url "$RPC" 2>&1 | grep -E '✓|Error|error' || true

  sleep 2
done

# ── Step 3: Output env vars ─────────────────────────────────────────────────

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Step 3: Environment Variables"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Map app names to env var names
typeset -A env_names
env_names=(
  [escrow]="NEXT_PUBLIC_ESCROW_LOOM_ID"
  [treasury]="NEXT_PUBLIC_TREASURY_LOOM_ID"
  [vesting]="NEXT_PUBLIC_VESTING_LOOM_ID"
  [launchpad]="NEXT_PUBLIC_LAUNCHPAD_LOOM_ID"
  [splitter]="NEXT_PUBLIC_SPLITTER_LOOM_ID"
  [crowdfund]="NEXT_PUBLIC_CROWDFUND_LOOM_ID"
  [governance]="NEXT_PUBLIC_GOVERNANCE_LOOM_ID"
  [staking]="NEXT_PUBLIC_STAKING_LOOM_ID"
  [swap]="NEXT_PUBLIC_SWAP_LOOM_ID"
  [airdrop]="NEXT_PUBLIC_AIRDROP_LOOM_ID"
  [timelock]="NEXT_PUBLIC_TIMELOCK_LOOM_ID"
)

# Write to file and print
> "$OUTPUT_FILE"
for app_name in escrow treasury vesting launchpad splitter crowdfund governance staking swap airdrop timelock; do
  loom_id="${loom_ids[$app_name]}"
  env_name="${env_names[$app_name]}"
  if [[ -n "$loom_id" ]]; then
    echo "${env_name}=${loom_id}" >> "$OUTPUT_FILE"
    echo "  ${env_name}=${loom_id}"
  fi
done

echo ""
echo "  Saved to: $OUTPUT_FILE"
echo ""
echo "  To update .env.local:"
echo "    cp $OUTPUT_FILE wallet/.env.local"
echo ""
echo "  To update apps-config.ts hardcoded defaults:"
echo "    Review and update wallet/lib/apps-config.ts"
echo ""
echo "Done! All 11 contracts deployed and uploaded."
echo ""
