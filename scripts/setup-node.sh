#!/usr/bin/env bash
set -e

# ---------------------------------------------------------------------------
# Norn Protocol -- Node Setup Script
# Automates installation of a Norn node, wallet creation, and devnet config.
# Safe to run multiple times (idempotent).
#
# Usage:
#   curl -sSf https://raw.githubusercontent.com/augmnt/norn-protocol/main/scripts/setup-node.sh | bash
#   curl -sSf ... | bash -s -- --start
#   curl -sSf ... | bash -s -- --systemd
#   curl -sSf ... | bash -s -- --start --faucet
#
# Options:
#   --start       Start the node in the background after setup
#   --systemd     Install and enable a systemd service (Linux only, requires sudo)
#   --faucet      Request devnet faucet funds after setup
#   --data-dir    Data directory (default: ~/.norn/data)
#   --help        Show this help message
# ---------------------------------------------------------------------------

# -- Options ----------------------------------------------------------------

OPT_START=false
OPT_SYSTEMD=false
OPT_FAUCET=false
OPT_DATA_DIR="$HOME/.norn/data"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --start)    OPT_START=true;   shift ;;
    --systemd)  OPT_SYSTEMD=true; shift ;;
    --faucet)   OPT_FAUCET=true;  shift ;;
    --data-dir) OPT_DATA_DIR="$2"; shift 2 ;;
    --help|-h)
      printf "Norn Protocol -- Node Setup Script\n\n"
      printf "Usage:\n"
      printf "  setup-node.sh [OPTIONS]\n\n"
      printf "Options:\n"
      printf "  --start       Start the node in the background after setup\n"
      printf "  --systemd     Install and enable a systemd service (Linux only)\n"
      printf "  --faucet      Request devnet faucet funds after setup\n"
      printf "  --data-dir    Data directory (default: ~/.norn/data)\n"
      printf "  --help        Show this help message\n"
      exit 0 ;;
    *) printf "Unknown option: %s\n" "$1"; exit 1 ;;
  esac
done

# -- Colors & symbols ------------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'

SYM_OK="${GREEN}✓${RESET}"
SYM_BULLET="${CYAN}●${RESET}"
SYM_DASH="${DIM}–${RESET}"

# -- Helpers ----------------------------------------------------------------

info()    { printf "  ${SYM_BULLET}  %b\n" "$*"; }
success() { printf "  ${SYM_OK}  %b\n" "$*"; }
warn()    { printf "  ${YELLOW}●${RESET}  %b\n" "$*"; }
fail()    { printf "  ${RED}●${RESET}  %b\n" "$*" >&2; exit 1; }
step()    { printf "\n${BOLD}%s${RESET}\n" "$*"; }

STEP_NUM=0
next_step() { STEP_NUM=$((STEP_NUM + 1)); step "${STEP_NUM}. $1"; }

# -- Platform detection -----------------------------------------------------

detect_platform() {
  case "$(uname -s)" in
    Darwin) PLATFORM="macos" ;;
    Linux)  PLATFORM="linux" ;;
    *)      PLATFORM="unknown" ;;
  esac
}

# -- Header -----------------------------------------------------------------

print_header() {
  printf "\n"
  printf "${BOLD}${CYAN}"
  printf "  _   _  ___  ____  _   _\n"
  printf " | \\ | |/ _ \\|  _ \\| \\ | |\n"
  printf " |  \\| | | | | |_) |  \\| |\n"
  printf " | |\\  | |_| |  _ <| |\\  |\n"
  printf " |_| \\_|\\___/|_| \\_\\_| \\_|\n"
  printf "${RESET}\n"
  printf "  ${DIM}Node Setup Script${RESET}\n"
  printf "  ${DIM}------------------------------------------${RESET}\n"
  printf "\n"

  detect_platform

  local opts=""
  $OPT_START && opts="${opts} --start"
  $OPT_SYSTEMD && opts="${opts} --systemd"
  $OPT_FAUCET && opts="${opts} --faucet"
  if [ -n "$opts" ]; then
    info "Options:${BOLD}${opts}${RESET}"
  fi
  info "Platform: ${BOLD}${PLATFORM}${RESET}"
  info "Data dir: ${DIM}${OPT_DATA_DIR}${RESET}"
}

# -- Prerequisite checks ----------------------------------------------------

check_cargo() {
  next_step "Checking prerequisites"

  if command -v cargo &>/dev/null; then
    local cargo_version
    cargo_version="$(cargo --version 2>/dev/null)"
    success "Cargo found: ${DIM}${cargo_version}${RESET}"
  else
    warn "Cargo not found -- installing Rust via rustup"

    if ! command -v curl &>/dev/null; then
      fail "curl is required to install Rust. Please install curl first."
    fi

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

    if [ -f "$HOME/.cargo/env" ]; then
      # shellcheck disable=SC1091
      source "$HOME/.cargo/env"
    fi

    if ! command -v cargo &>/dev/null; then
      fail "Rust installation completed but cargo is still not in PATH. Please restart your shell and re-run this script."
    fi

    success "Rust installed: ${DIM}$(cargo --version)${RESET}"
  fi

  # Check for git (needed for cargo install --git)
  if command -v git &>/dev/null; then
    success "Git found: ${DIM}$(git --version)${RESET}"
  else
    fail "git is required. Please install git first."
  fi
}

check_platform_deps() {
  if [ "$PLATFORM" = "macos" ]; then
    if ! xcode-select -p &>/dev/null; then
      warn "Xcode command-line tools not found -- installing"
      xcode-select --install 2>/dev/null || true
      info "If prompted, please accept the Xcode license and re-run this script."
    else
      success "Xcode command-line tools found"
    fi
  elif [ "$PLATFORM" = "linux" ]; then
    if ! command -v cc &>/dev/null; then
      warn "C compiler not found -- attempting to install build-essential"
      if command -v apt-get &>/dev/null; then
        sudo apt-get update -qq && sudo apt-get install -y -qq build-essential pkg-config libssl-dev
        success "Build tools installed"
      elif command -v yum &>/dev/null; then
        sudo yum groupinstall -y "Development Tools" && sudo yum install -y openssl-devel
        success "Build tools installed"
      else
        warn "Could not auto-install. Please install a C compiler manually."
      fi
    else
      success "C toolchain found"
    fi
  fi

  if [ "$OPT_SYSTEMD" = true ] && [ "$PLATFORM" != "linux" ]; then
    warn "--systemd is only supported on Linux. Ignoring."
    OPT_SYSTEMD=false
  fi
}

# -- Install norn-node ------------------------------------------------------

install_norn() {
  next_step "Installing norn-node"

  if command -v norn &>/dev/null; then
    local current_version
    current_version="$(norn --version 2>/dev/null || echo 'unknown')"
    success "norn is already installed: ${DIM}${current_version}${RESET}"

    printf "\n"
    printf "  ${SYM_DASH}  Reinstall / update? [y/N] "
    read -r reinstall </dev/tty
    if [[ ! "$reinstall" =~ ^[Yy]$ ]]; then
      info "Skipping installation"
      return 0
    fi
  fi

  info "Building norn-node from source (this may take a few minutes)..."
  cargo install --git https://github.com/augmnt/norn-protocol norn-node
  success "norn-node installed: ${DIM}$(norn --version 2>/dev/null)${RESET}"
}

# -- Wallet setup -----------------------------------------------------------

setup_wallet() {
  next_step "Wallet setup"

  local wallet_dir="$HOME/.norn/wallets"

  if [ -d "$wallet_dir" ] && [ "$(ls -A "$wallet_dir" 2>/dev/null)" ]; then
    local count
    count="$(ls -1 "$wallet_dir" 2>/dev/null | wc -l | tr -d ' ')"
    success "${count} wallet(s) found in ${DIM}${wallet_dir}${RESET}"
    return 0
  fi

  info "No wallets found in ${DIM}${wallet_dir}${RESET}"
  printf "\n"
  printf "  ${SYM_DASH}  Would you like to create a new wallet? [Y/n] "
  read -r create_wallet </dev/tty

  if [[ "$create_wallet" =~ ^[Nn]$ ]]; then
    warn "Skipping wallet creation. You can create one later with:"
    info "  norn wallet create --name validator"
    return 0
  fi

  info "Creating wallet 'validator'..."
  norn wallet create --name validator
  success "Wallet created"
}

# -- Devnet configuration ---------------------------------------------------

configure_network() {
  next_step "Configuring network"

  norn wallet config --network dev
  success "Network set to ${BOLD}dev${RESET}"

  # Create data directory
  mkdir -p "$OPT_DATA_DIR"
  success "Data directory ready: ${DIM}${OPT_DATA_DIR}${RESET}"
}

# -- Faucet -----------------------------------------------------------------

request_faucet() {
  if [ "$OPT_FAUCET" = false ]; then
    return 0
  fi

  next_step "Requesting faucet funds"

  # Faucet needs the node running; if we're about to start it, wait
  if [ "$OPT_START" = true ] || [ "$OPT_SYSTEMD" = true ]; then
    info "Waiting for node to start before requesting faucet..."
    local attempts=0
    while ! norn wallet node-info --rpc-url http://localhost:9741 &>/dev/null; do
      attempts=$((attempts + 1))
      if [ $attempts -gt 30 ]; then
        warn "Node not responding after 30s. You can request faucet manually:"
        info "  norn wallet faucet"
        return 0
      fi
      sleep 1
    done
    success "Node is responding"
  fi

  if norn wallet faucet 2>/dev/null; then
    success "Faucet funds received"
  else
    warn "Faucet request failed. You can try again later:"
    info "  norn wallet faucet"
  fi
}

# -- Start node (background) -----------------------------------------------

start_node_background() {
  if [ "$OPT_START" = false ]; then
    return 0
  fi

  # Don't start in background if systemd will handle it
  if [ "$OPT_SYSTEMD" = true ]; then
    return 0
  fi

  next_step "Starting node"

  # Check if already running
  if pgrep -f "norn run" &>/dev/null; then
    success "Node is already running"
    return 0
  fi

  local log_file="$HOME/.norn/norn-node.log"
  mkdir -p "$HOME/.norn"

  info "Starting node in background..."
  nohup norn run --dev --data-dir "$OPT_DATA_DIR" > "$log_file" 2>&1 &
  local pid=$!

  # Wait briefly and check it's still running
  sleep 2
  if kill -0 "$pid" 2>/dev/null; then
    success "Node started (PID ${DIM}${pid}${RESET})"
    info "Logs: ${DIM}${log_file}${RESET}"
    info "Stop: ${DIM}kill ${pid}${RESET}"

    # Save PID for convenience
    echo "$pid" > "$HOME/.norn/node.pid"
  else
    warn "Node may have failed to start. Check logs:"
    info "  ${DIM}cat ${log_file}${RESET}"
  fi
}

# -- Systemd service --------------------------------------------------------

setup_systemd() {
  if [ "$OPT_SYSTEMD" = false ]; then
    return 0
  fi

  next_step "Setting up systemd service"

  local norn_bin
  norn_bin="$(command -v norn)"
  if [ -z "$norn_bin" ]; then
    fail "norn binary not found in PATH"
  fi

  local service_file="/etc/systemd/system/norn-node.service"

  # Check if service already exists and is running
  if systemctl is-active --quiet norn-node 2>/dev/null; then
    success "norn-node service is already running"
    printf "\n"
    printf "  ${SYM_DASH}  Restart with updated config? [y/N] "
    read -r restart_svc </dev/tty
    if [[ ! "$restart_svc" =~ ^[Yy]$ ]]; then
      info "Keeping existing service"
      return 0
    fi
  fi

  # Create norn user if it doesn't exist
  if ! id norn &>/dev/null; then
    info "Creating 'norn' system user..."
    sudo useradd --system --home /var/lib/norn --shell /usr/sbin/nologin norn 2>/dev/null || true
    success "User 'norn' created"
  else
    success "User 'norn' exists"
  fi

  # Create data directory
  sudo mkdir -p /var/lib/norn/data
  sudo chown -R norn:norn /var/lib/norn

  # Copy wallet files so the norn user has access
  if [ -d "$HOME/.norn/wallets" ] && [ "$(ls -A "$HOME/.norn/wallets" 2>/dev/null)" ]; then
    sudo mkdir -p /var/lib/norn/.norn/wallets
    sudo cp -n "$HOME/.norn/wallets/"* /var/lib/norn/.norn/wallets/ 2>/dev/null || true
    sudo chown -R norn:norn /var/lib/norn/.norn
    success "Wallet files copied to service user"
  fi

  # Write service file
  info "Writing ${DIM}${service_file}${RESET}"
  sudo tee "$service_file" > /dev/null << SERVICEEOF
[Unit]
Description=Norn Protocol Node
Documentation=https://norn.network/docs/run-a-node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=norn
Group=norn
ExecStart=${norn_bin} run --dev --storage sqlite --data-dir /var/lib/norn/data
Restart=always
RestartSec=5
LimitNOFILE=65535
Environment=HOME=/var/lib/norn

[Install]
WantedBy=multi-user.target
SERVICEEOF

  success "Service file written"

  # Enable and start
  sudo systemctl daemon-reload
  sudo systemctl enable norn-node
  sudo systemctl start norn-node

  # Verify
  sleep 2
  if systemctl is-active --quiet norn-node; then
    success "norn-node service is ${GREEN}active${RESET}"
    info "Status: ${DIM}sudo systemctl status norn-node${RESET}"
    info "Logs:   ${DIM}sudo journalctl -u norn-node -f${RESET}"
    info "Stop:   ${DIM}sudo systemctl stop norn-node${RESET}"
  else
    warn "Service may have failed to start. Check:"
    info "  sudo systemctl status norn-node"
    info "  sudo journalctl -u norn-node --no-pager -n 20"
  fi
}

# -- Completion message -----------------------------------------------------

print_complete() {
  printf "\n"
  printf "  ${GREEN}${BOLD}------------------------------------------${RESET}\n"
  printf "  ${GREEN}${BOLD}  Setup complete${RESET}\n"
  printf "  ${GREEN}${BOLD}------------------------------------------${RESET}\n"
  printf "\n"

  # Show status summary
  if [ "$OPT_SYSTEMD" = true ]; then
    printf "  ${SYM_OK}  Node running as systemd service\n"
  elif [ "$OPT_START" = true ]; then
    printf "  ${SYM_OK}  Node running in background\n"
  fi
  if [ "$OPT_FAUCET" = true ]; then
    printf "  ${SYM_OK}  Faucet funds requested\n"
  fi

  # Show next steps only for things not already done
  local has_next=false

  if [ "$OPT_START" = false ] && [ "$OPT_SYSTEMD" = false ]; then
    has_next=true
  fi
  if [ "$OPT_FAUCET" = false ]; then
    has_next=true
  fi

  if [ "$has_next" = true ]; then
    printf "\n"
    printf "  ${BOLD}Next steps:${RESET}\n"

    if [ "$OPT_START" = false ] && [ "$OPT_SYSTEMD" = false ]; then
      printf "\n"
      printf "  ${SYM_DASH}  Start the node:\n"
      printf "     ${CYAN}norn run --dev${RESET}\n"
    fi

    if [ "$OPT_FAUCET" = false ]; then
      printf "\n"
      printf "  ${SYM_DASH}  Get devnet funds:\n"
      printf "     ${CYAN}norn wallet faucet${RESET}\n"
    fi
  fi

  printf "\n"
  printf "  ${BOLD}Useful commands:${RESET}\n"
  printf "\n"
  printf "  ${SYM_DASH}  Check rewards:    ${CYAN}norn wallet rewards${RESET}\n"
  printf "  ${SYM_DASH}  View dashboard:   ${CYAN}norn wallet whoami${RESET}\n"
  printf "  ${SYM_DASH}  Node status:      ${CYAN}norn wallet node-info${RESET}\n"
  printf "  ${SYM_DASH}  Validator set:    ${CYAN}norn wallet validators${RESET}\n"
  printf "  ${SYM_DASH}  Documentation:    ${CYAN}https://norn.network/docs/run-a-node${RESET}\n"
  printf "\n"
}

# -- Main -------------------------------------------------------------------

main() {
  print_header
  check_cargo
  check_platform_deps
  install_norn
  setup_wallet
  configure_network

  # Start node before faucet (faucet needs a running node)
  start_node_background
  setup_systemd
  request_faucet

  print_complete
}

main
