#!/usr/bin/env bash
# Deploy all 11 app contracts to the running devnet node.
#
# Usage:
#   ./scripts/deploy-contracts.sh
#
# Prerequisites:
#   - norn node running at 127.0.0.1:9741 (or RPC_URL env var)
#   - NORN_WALLET_PASSWORD set (or will prompt for each)
#   - Active wallet has sufficient NORN (11 * 100 NORN deploy fee = 1100 NORN)
#   - WASMs already built: cargo build --release --target wasm32-unknown-unknown
#
# Output:
#   - Deploys all 11 contracts and captures loom IDs
#   - Updates wallet/.env.local and wallet/lib/apps-config.ts with new loom IDs

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
WALLET_DIR="$PROJECT_DIR/wallet"
RPC_FLAG=""
if [ -n "${RPC_URL:-}" ]; then
  RPC_FLAG="--rpc-url $RPC_URL"
fi

# Contract name -> WASM binary name mapping
declare -A CONTRACTS=(
  ["escrow"]="escrow"
  ["treasury"]="multisig_treasury"
  ["vesting"]="vesting"
  ["launchpad"]="launchpad"
  ["splitter"]="splitter"
  ["crowdfund"]="crowdfund"
  ["governance"]="governance"
  ["staking"]="staking"
  ["swap"]="swap"
  ["airdrop"]="airdrop"
  ["timelock"]="timelock"
)

# Contract name -> directory name mapping (for WASM path)
declare -A DIRS=(
  ["escrow"]="escrow"
  ["treasury"]="multisig-treasury"
  ["vesting"]="vesting"
  ["launchpad"]="launchpad"
  ["splitter"]="splitter"
  ["crowdfund"]="crowdfund"
  ["governance"]="governance"
  ["staking"]="staking"
  ["swap"]="swap"
  ["airdrop"]="airdrop"
  ["timelock"]="timelock"
)

# Collect deployed loom IDs
declare -A LOOM_IDS

echo ""
echo "  ╔══════════════════════════════════════╗"
echo "  ║     Deploy All App Contracts         ║"
echo "  ╚══════════════════════════════════════╝"
echo ""

# Check node connectivity
echo "  Checking node connectivity..."
if ! norn wallet node-info $RPC_FLAG > /dev/null 2>&1; then
  echo "  ERROR: Cannot connect to node. Is it running?"
  echo "  Start with: norn run --dev --reset-state"
  exit 1
fi
echo "  Node is reachable."
echo ""

# Deploy order
ORDER="escrow treasury vesting launchpad splitter crowdfund governance staking swap airdrop timelock"

for name in $ORDER; do
  wasm_name="${CONTRACTS[$name]}"
  dir_name="${DIRS[$name]}"
  wasm_path="$PROJECT_DIR/examples/$dir_name/target/wasm32-unknown-unknown/release/$wasm_name.wasm"

  if [ ! -f "$wasm_path" ]; then
    echo "  SKIP: $name — WASM not found at $wasm_path"
    echo "  Build with: cd examples/$dir_name && cargo build --release --target wasm32-unknown-unknown"
    continue
  fi

  echo "  Deploying $name..."

  # Deploy the loom (creates entry, returns loom ID in output)
  output=$(norn wallet deploy-loom --name "$name" --yes $RPC_FLAG 2>&1) || {
    echo "  FAILED to deploy $name:"
    echo "$output" | sed 's/^/    /'
    continue
  }

  # Extract loom ID from output (line containing "Loom ID: <hex>")
  loom_id=$(echo "$output" | grep -o 'Loom ID: [0-9a-f]\{64\}' | head -1 | awk '{print $3}')

  if [ -z "$loom_id" ]; then
    echo "  FAILED to extract loom ID for $name. Output:"
    echo "$output" | sed 's/^/    /'
    continue
  fi

  echo "    Loom ID: $loom_id"

  # Upload bytecode
  echo "    Uploading bytecode ($(stat -f%z "$wasm_path" 2>/dev/null || stat --printf="%s" "$wasm_path" 2>/dev/null) bytes)..."
  upload_output=$(norn wallet upload-bytecode --loom-id "$loom_id" --bytecode "$wasm_path" $RPC_FLAG 2>&1) || {
    echo "    FAILED to upload bytecode for $name:"
    echo "$upload_output" | sed 's/^/      /'
    continue
  }

  if echo "$upload_output" | grep -q "uploaded and initialized"; then
    echo "    Bytecode uploaded successfully."
  else
    echo "    Upload output:"
    echo "$upload_output" | sed 's/^/      /'
  fi

  LOOM_IDS[$name]="$loom_id"
  echo ""
done

echo "  ─────────────────────────────────────"
echo "  Deployed ${#LOOM_IDS[@]} / 11 contracts"
echo ""

if [ ${#LOOM_IDS[@]} -eq 0 ]; then
  echo "  No contracts deployed. Exiting."
  exit 1
fi

# Print all loom IDs
echo "  Loom IDs:"
for name in $ORDER; do
  if [ -n "${LOOM_IDS[$name]:-}" ]; then
    upper=$(echo "$name" | tr '[:lower:]' '[:upper:]')
    echo "    $upper: ${LOOM_IDS[$name]}"
  fi
done
echo ""

# Update wallet/.env.local
ENV_FILE="$WALLET_DIR/.env.local"
echo "  Updating $ENV_FILE ..."

{
  for name in $ORDER; do
    if [ -n "${LOOM_IDS[$name]:-}" ]; then
      upper=$(echo "$name" | tr '[:lower:]' '[:upper:]')
      echo "NEXT_PUBLIC_${upper}_LOOM_ID=${LOOM_IDS[$name]}"
    fi
  done
} > "$ENV_FILE"

echo "  Updated .env.local with ${#LOOM_IDS[@]} loom IDs."

# Update wallet/lib/apps-config.ts — replace the hardcoded hex strings
CONFIG_FILE="$WALLET_DIR/lib/apps-config.ts"
echo "  Updating $CONFIG_FILE ..."

for name in $ORDER; do
  if [ -n "${LOOM_IDS[$name]:-}" ]; then
    upper=$(echo "$name" | tr '[:lower:]' '[:upper:]')
    # Replace the hardcoded hex fallback in the || "..." part
    if [[ "$OSTYPE" == "darwin"* ]]; then
      sed -i '' "s/export const ${upper}_LOOM_ID = .*/export const ${upper}_LOOM_ID = process.env.NEXT_PUBLIC_${upper}_LOOM_ID || \"${LOOM_IDS[$name]}\";/" "$CONFIG_FILE"
    else
      sed -i "s/export const ${upper}_LOOM_ID = .*/export const ${upper}_LOOM_ID = process.env.NEXT_PUBLIC_${upper}_LOOM_ID || \"${LOOM_IDS[$name]}\";/" "$CONFIG_FILE"
    fi
  fi
done

echo "  Updated apps-config.ts with new loom IDs."
echo ""
echo "  Done! Remember to commit the updated config files."
echo ""
