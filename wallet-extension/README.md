# Norn Wallet

Chrome browser extension for the Norn Protocol. Send and receive NORN, manage accounts, register NornNames, and browse tokens — all from your browser toolbar.

Built with Vite, React 19, Zustand, shadcn/ui, and the `@norn-protocol/sdk`.

## Features

- **Account management** — Create new wallets or import existing ones (private key hex or CLI export)
- **Send & receive NORN** — Transfer native NORN tokens with memo support and QR code receiving
- **NornName resolution** — Send to registered names (e.g. `alice`) instead of hex addresses
- **Token browser** — View all NT-1 tokens on the network with live balances
- **Transaction history** — Browse recent sends and receives with status indicators
- **NornName registration** — Register names directly from the extension (1 NORN fee)
- **Multi-account support** — Switch between accounts without re-entering passwords
- **Auto-lock** — Wallet locks automatically after inactivity; re-authentication required on popup reopen
- **CLI wallet import** — Import wallets exported from the `norn wallet` CLI tool

## Prerequisites

- **Node.js** 18+ (20 recommended)
- **npm** 9+
- **Chrome** or Chromium-based browser (Brave, Edge, Arc, etc.)
- Access to a running Norn node (local or remote)

## Quick Start

### 1. Build the TypeScript SDK

The extension depends on the local `@norn-protocol/sdk`:

```bash
cd sdk/typescript && npm run build && cd ../../wallet-extension
```

### 2. Install dependencies

```bash
npm install
```

### 3. Build the extension

```bash
npm run build
```

### 4. Load in Chrome

1. Open `chrome://extensions`
2. Enable **Developer mode** (top right toggle)
3. Click **Load unpacked**
4. Select the `wallet-extension/dist` directory

The Norn Wallet icon appears in your browser toolbar.

## Development

For hot-reload during development:

```bash
npm run dev
```

Then load the `wallet-extension/dist` directory in Chrome as above. The extension will rebuild on file changes — reload the extension in `chrome://extensions` to pick up updates.

## Network Configuration

By default, the wallet connects to the devnet seed node. To change the RPC endpoint:

1. Open the wallet popup
2. Go to **Settings**
3. Update the **RPC URL** field

| Environment | RPC URL |
|-------------|---------|
| Local node | `http://localhost:9944` |
| Devnet seed | `http://164.90.182.133:9741` |

## Pages

| Page | Description |
|------|-------------|
| Welcome | First-run screen with options to create, import, or import from CLI |
| Create Wallet | Generate a new wallet with password encryption + backup key display |
| Import Wallet | Import an existing wallet from a 64-character private key hex |
| Import from CLI | Step-by-step guide to import a wallet exported from the `norn` CLI |
| Unlock | Password entry screen shown when the wallet is locked |
| Dashboard | Balance display, quick actions (Send, Receive, Faucet, Names), recent activity |
| Send | Transfer NORN to an address or NornName with amount and optional memo |
| Confirm | Review transaction details before signing and submitting |
| Receive | QR code and copyable address for receiving NORN |
| Activity | Full transaction history with send/receive indicators |
| Tokens | Browse all NT-1 tokens on the network with balances |
| Accounts | Switch between accounts, create new ones, or import additional wallets |
| Register Name | Register NornNames and view owned names |
| Settings | Network configuration, wallet export, and lock controls |

## Importing a CLI Wallet

If you have a wallet created with the `norn wallet` CLI, you can import it into the extension:

1. In your terminal, export the private key:
   ```bash
   norn wallet export <wallet-name> --show-private-key
   ```
2. Open the extension and choose **Import from CLI** on the welcome screen (or from the Accounts page)
3. Paste the 64-character hex key
4. Set an account name and password
5. The wallet is now available in the extension

## Project Structure

```
wallet-extension/
  manifest.json         # Chrome MV3 manifest
  src/
    background/         # Service worker (auto-lock alarms)
    popup/
      pages/            # 14 page components
      components/
        ui/             # shadcn/ui base + custom components
        layout/         # Header, BottomNav, PageTransition
        wallet/         # BalanceCard, ActivityRow, AccountPill, TokenRow
    stores/             # Zustand state (wallet, navigation, network)
    lib/                # RPC client, keystore, formatters, config
    types/              # Route and account type definitions
```

## Security

- Private keys are encrypted with a user-provided password using the Web Crypto API (PBKDF2 + AES-GCM)
- Keys exist in memory only while the wallet is unlocked — closing the popup clears them
- The wallet auto-locks after a configurable inactivity period
- No private key material is ever sent over the network
- The extension requests only `storage` and `alarms` permissions

## Troubleshooting

**Extension not loading** — Make sure you built with `npm run build` and selected the `dist` directory (not the project root) when loading unpacked.

**"Network error" on dashboard** — The RPC endpoint is unreachable. Check Settings to verify the URL, and ensure a Norn node is running at that address.

**"Wallet is locked" after reopening** — This is expected. The popup's in-memory state is cleared when it closes. Enter your password to unlock.

**Transaction stuck** — The node must be producing blocks. If running a local dev node, ensure it was started with `norn run --dev`.
