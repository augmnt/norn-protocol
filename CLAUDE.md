# Norn Protocol — Claude Code Instructions

## Project Overview

Rust workspace (11 crates) implementing a thread-based L1 blockchain. Includes a TypeScript SDK (`sdk/typescript/`), a Next.js block explorer (`explorer/`), and a Next.js website (`website/`).

## Build & Verify

```bash
# Rust — must all pass before any PR
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check

# TypeScript SDK
cd sdk/typescript && npm test && npx tsc --noEmit

# Explorer
cd explorer && npm run build

# Website
cd website && npm run build
```

## Design System

See `DESIGN.md` for the complete specification. The rules below are the critical subset for generating code.

### Web UI (Explorer & Website)

**Dark mode is enforced.** Never add light mode toggles or conditional light/dark styling. Use `forcedTheme="dark"` in ThemeProvider.

**Stack — do not deviate:**
- Next.js 15 + React 19
- Tailwind CSS with `tailwindcss-animate` plugin
- shadcn/ui components (Radix primitives + CVA)
- lucide-react for all icons
- Inter (sans) + JetBrains Mono (mono) from Google Fonts
- `cn()` helper from `clsx` + `tailwind-merge` for class merging

**Colors — use semantic tokens, never raw values:**
- Brand accent: `text-norn`, `bg-norn`, `hover:bg-norn/90`
- Text: `text-foreground` (primary), `text-muted-foreground` (secondary)
- Backgrounds: `bg-background`, `bg-card`, `bg-muted`, `bg-secondary`
- Borders: `border-border`
- Destructive: `text-destructive`, `bg-destructive`
- Never use raw colors like `text-blue-500` or `bg-gray-800`

**Typography rules:**
- Addresses, hashes, amounts, code: always `font-mono`
- Table headers / stat labels: `text-xs uppercase tracking-wider text-muted-foreground`
- Numeric values in tables/stats: `tabular-nums`
- Page titles: `text-heading font-semibold`

**Component conventions:**
- Buttons: use CVA variants (`default`, `outline`, `ghost`, `secondary`, `norn`, `destructive`). Sizes: `default`, `sm`, `lg`, `icon`
- Cards: use shadcn Card with CardHeader/CardTitle/CardDescription/CardContent
- Page layout: wrap content in `PageContainer` (`max-w-7xl` with responsive padding)
- Empty states: use `EmptyState` component with lucide icon
- Tables: use `DataTable` with `text-xs uppercase tracking-wider` headers, `hover:bg-muted/50` rows
- Links to internal pages: `text-norn hover:underline`
- Badges: use `norn` variant for protocol-specific labels (`bg-norn/10 text-norn`)
- Toasts: Sonner, dark theme, bottom-right position

**Spacing:**
- Page container: `px-4 sm:px-6 lg:px-8 py-6`
- Between sections: `space-y-6` or `gap-6`
- Card padding: `p-6` (content `p-6 pt-0`)
- Stat cards: `p-4`

**Animations:**
- Entrance: `animate-fade-in` (0.4s) or `animate-slide-in` (0.2s)
- Stat values: `animate-count-up` (0.3s)
- All transitions: ease-out, 0.2–0.5s. Never bouncy or playful

**Responsive:**
- Mobile-first, breakpoints: `sm:` `md:` `lg:` `xl:`
- Hide non-essential columns on mobile: `hidden sm:table-cell`
- Navigation: hidden on mobile with menu toggle

### Terminal UI (Wallet CLI)

**Output rules:**
- 2-space indent for all output lines
- Success: `print_success(msg)` — green `✓` prefix
- Errors: `print_error(msg, hint)` — red `Error:` prefix, optional dim hint
- Dividers: `print_divider()` — 32-char dim line
- Leading and trailing blank lines around command output

**Tables (comfy_table):**
- List data: `data_table(&["Header", ...])` — UTF8_FULL preset
- Key-value: `info_table()` — NOTHING preset (no borders)
- Print with `print_table()` for consistent 2-space indent
- Amounts: `cell_right()`. Success: `cell_green()`. Warning: `cell_yellow()`

**Symbols:** Only `✓`, `●`, `–`, and box-drawing characters. Never emoji.

**Addresses:** Always `0x`-prefixed, truncate with `truncate_hex_string(s, half_len)`.

**Amounts:** Use `format_amount()` for NORN, `format_token_amount(amount, decimals)` for custom tokens. Comma-grouped, trailing zeros stripped.

**Prompts (dialoguer):** `prompt_password()`, `prompt_new_password()`, `confirm()`.

### Cross-Surface Rules

1. Brand color is `hsl(210, 12%, 49%)` — "norn blue". The only accent color
2. Monospace for all machine-readable data (addresses, hashes, amounts, code)
3. Green = success/received, yellow/amber = warning/sent, red = error, cyan = info
4. Minimal, no decoration, no gradients, no illustrations, no emoji in web UI
5. Subtle entrance animations only (0.2–0.5s ease-out). No bouncy/spring physics

## Rust Conventions

- `borsh` for serialization, `blake3` for hashing, `ed25519-dalek` for signatures
- Error enums use `thiserror`
- `norn-types` is shared types only (no logic)
- `Amount` = `u128`, `Address` = `[u8; 20]`
- Storage: `KvStore` trait with memory/SQLite/RocksDB backends
- Always `auto_register_if_needed()` BOTH `from` AND `to` before `apply_transfer()`
- `norn-node` has both `lib.rs` and `main.rs` — new modules must be in both
- borsh v1.x: `borsh::to_vec(&val)` not `.try_to_vec()`
- wasmtime: `default-features = false, features = ["cranelift", "runtime"]`

## Git

- Follow Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`
- Never force push to main
