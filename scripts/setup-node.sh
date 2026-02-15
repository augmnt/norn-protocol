#!/usr/bin/env bash
set -e

# ---------------------------------------------------------------------------
# Norn Protocol -- Node Setup Script
# Automates installation of a Norn node, wallet creation, and devnet config.
# Safe to run multiple times (idempotent).
# ---------------------------------------------------------------------------

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
}

# -- Prerequisite checks ----------------------------------------------------

check_cargo() {
  step "1. Checking prerequisites"

  if command -v cargo &>/dev/null; then
    local cargo_version
    cargo_version="$(cargo --version 2>/dev/null)"
    success "Cargo found: ${DIM}${cargo_version}${RESET}"
    return 0
  fi

  warn "Cargo not found -- installing Rust via rustup"

  if ! command -v curl &>/dev/null; then
    fail "curl is required to install Rust. Please install curl first."
  fi

  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

  # Source the cargo environment so it is available for the rest of this script
  if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  fi

  if ! command -v cargo &>/dev/null; then
    fail "Rust installation completed but cargo is still not in PATH. Please restart your shell and re-run this script."
  fi

  success "Rust installed: ${DIM}$(cargo --version)${RESET}"
}

check_platform_deps() {
  detect_platform

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
      warn "C compiler not found. You may need to install build-essential:"
      info "  sudo apt-get install -y build-essential pkg-config libssl-dev"
    else
      success "C toolchain found"
    fi
  fi
}

# -- Install norn-node ------------------------------------------------------

install_norn() {
  step "2. Installing norn-node"

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
  success "norn-node installed successfully"
}

# -- Wallet setup -----------------------------------------------------------

setup_wallet() {
  step "3. Wallet setup"

  local wallet_dir="$HOME/.norn/wallets"

  if [ -d "$wallet_dir" ] && [ "$(ls -A "$wallet_dir" 2>/dev/null)" ]; then
    success "Existing wallet(s) found in ${DIM}${wallet_dir}${RESET}"
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

configure_devnet() {
  step "4. Configuring for devnet"

  norn wallet config --network dev
  success "Network set to ${BOLD}dev${RESET}"
}

# -- Completion message -----------------------------------------------------

print_complete() {
  printf "\n"
  printf "  ${GREEN}${BOLD}------------------------------------------${RESET}\n"
  printf "  ${GREEN}${BOLD}  Setup complete${RESET}\n"
  printf "  ${GREEN}${BOLD}------------------------------------------${RESET}\n"
  printf "\n"
  printf "  ${BOLD}Next steps:${RESET}\n"
  printf "\n"
  printf "  ${SYM_DASH}  Start the node:\n"
  printf "     ${CYAN}norn run --dev${RESET}\n"
  printf "\n"
  printf "  ${SYM_DASH}  Get devnet funds:\n"
  printf "     ${CYAN}norn wallet faucet${RESET}\n"
  printf "\n"
  printf "  ${SYM_DASH}  Check rewards:\n"
  printf "     ${CYAN}norn wallet rewards${RESET}\n"
  printf "\n"
  printf "  ${SYM_DASH}  View dashboard:\n"
  printf "     ${CYAN}norn wallet whoami${RESET}\n"
  printf "\n"
}

# -- Main -------------------------------------------------------------------

main() {
  print_header
  check_cargo
  check_platform_deps
  install_norn
  setup_wallet
  configure_devnet
  print_complete
}

main "$@"
