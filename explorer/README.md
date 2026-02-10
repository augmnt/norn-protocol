# Norn Explorer

Block explorer for the Norn Protocol. Built with Next.js 15, shadcn/ui, and the `@norn-protocol/sdk`.

## Prerequisites

- **Node.js** 18+ (20 recommended)
- **npm** 9+
- Access to a running Norn node (local or remote)

## Quick Start

### 1. Install dependencies

```bash
cd explorer
npm install
```

> The `@norn-protocol/sdk` is linked locally from `../sdk/typescript`. Make sure the SDK is built before installing:
>
> ```bash
> cd ../sdk/typescript && npm run build && cd ../../explorer
> ```

### 2. Configure the RPC endpoint

Edit `.env.local` to point to your Norn node:

```env
NEXT_PUBLIC_RPC_URL=http://localhost:9944
NEXT_PUBLIC_WS_URL=ws://localhost:9944
NEXT_PUBLIC_CHAIN_NAME=Norn Devnet
```

**Common configurations:**

| Environment | RPC URL | WS URL |
|-------------|---------|--------|
| Local node | `http://localhost:9944` | `ws://localhost:9944` |
| Devnet seed | `http://164.90.182.133:9741` | `ws://164.90.182.133:9741` |

### 3. Run the dev server

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000).

## Running Against a Local Node

To run the explorer against a local Norn node:

1. **Start a local node** (from the repository root):

   ```bash
   cargo run -p norn-node -- run --dev
   ```

   This starts a solo dev node on `localhost:9944` with a pre-funded founder account.

2. **Update `.env.local`** to point to localhost:

   ```env
   NEXT_PUBLIC_RPC_URL=http://localhost:9944
   NEXT_PUBLIC_WS_URL=ws://localhost:9944
   NEXT_PUBLIC_CHAIN_NAME=Norn Local
   ```

3. **Start the explorer**:

   ```bash
   npm run dev
   ```

## Running Against Devnet

The default `.env.local` is preconfigured to connect to the devnet seed node. Just install and run:

```bash
npm install
npm run dev
```

## Production Build

```bash
npm run build
npm run start
```

The production build uses standalone output mode for containerized deployments.

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `NEXT_PUBLIC_RPC_URL` | `http://164.90.182.133:9741` | HTTP JSON-RPC endpoint |
| `NEXT_PUBLIC_WS_URL` | `ws://164.90.182.133:9741` | WebSocket endpoint for subscriptions |
| `NEXT_PUBLIC_CHAIN_NAME` | `Norn Devnet` | Display name for the network |

## Project Structure

```
explorer/
  app/              # Next.js App Router pages (10 routes)
  components/
    ui/             # shadcn/ui base + custom components
    layout/         # Header, footer, theme toggle
    search/         # Cmd+K search dialog
    dashboard/      # Dashboard widgets
    transactions/   # Transaction feed components
  hooks/            # React Query data-fetching hooks
  lib/              # Config, RPC client, formatters, utilities
  providers/        # React context providers (query, theme, WS)
  stores/           # Zustand real-time state
  types/            # Type re-exports from SDK
```

## Pages

| Route | Description |
|-------|-------------|
| `/` | Dashboard with network stats, recent blocks, recent transactions |
| `/blocks` | Paginated block list |
| `/block/[height]` | Block detail with metadata and activity counts |
| `/transactions` | Live transaction feed + pending transactions |
| `/address/[address]` | Account balances, transaction history, registered names |
| `/tokens` | Token registry |
| `/token/[tokenId]` | Token detail with supply info and live events |
| `/contracts` | Deployed contracts list |
| `/contract/[loomId]` | Contract detail with live execution events |
| `/validators` | Validator set and staking information |

## Troubleshooting

**`ERR_CONNECTION_REFUSED`** — The RPC endpoint is unreachable. Verify:
- The Norn node is running and listening on the configured port
- The URL in `.env.local` is correct
- No firewall is blocking the connection

**WebSocket errors** — The WS URL must match the same host/port as the RPC URL. The explorer uses WebSocket subscriptions for real-time block and transaction updates.

**SDK link errors** — If `npm install` fails on the SDK link, ensure the TypeScript SDK is built:
```bash
cd ../sdk/typescript && npm run build
```
