# Norn Protocol Design System

This document defines the visual language shared across all Norn Protocol surfaces: the **website** (norn.network), the **explorer** (explorer.norn.network), and the **wallet CLI**. Any new UI surface must follow these guidelines.

---

## Brand Identity

**Design philosophy:** Minimalist, professional, monochrome-first with a single accent color. Dark mode is enforced across all web surfaces. The aesthetic is technical and precise — no decorative elements, no gradients, no illustration. Information density over ornamentation.

**Brand color — Norn Blue:**
```
HSL:  210 12% 49%
Hex:  ~#6d7a8d
```

This is the only accent color. It appears as link color, branded buttons, status indicators, and chart primary. Both light and dark mode use the same value.

**Logo treatment:** The word "norn" in lowercase monospace (`font-mono`), letter-spacing `-0.02em`. In the terminal, the ANSI Shadow ASCII art in magenta+bold.

---

## Web Surfaces (Explorer & Website)

### Stack

| Layer | Technology |
|-------|-----------|
| Framework | Next.js 15, React 19 |
| Styling | Tailwind CSS 3.4 + `tailwindcss-animate` plugin |
| Components | shadcn/ui (Radix primitives + CVA variants) |
| Icons | lucide-react |
| Fonts | Inter (sans), JetBrains Mono (mono) — Google Fonts |
| State | React Query (server), Zustand (client/realtime) |
| Toasts | Sonner (dark theme, bottom-right) |
| Utilities | `cn()` from `clsx` + `tailwind-merge` |

### Color Tokens (CSS Variables, HSL format)

All colors are defined as HSL values without the `hsl()` wrapper, used as `hsl(var(--token))` in Tailwind config.

#### Dark Mode (enforced)

| Token | HSL Value | Usage |
|-------|-----------|-------|
| `--background` | `240 10% 3.9%` | Page background |
| `--foreground` | `0 0% 98%` | Primary text |
| `--card` | `240 10% 3.9%` | Card backgrounds |
| `--card-foreground` | `0 0% 98%` | Card text |
| `--popover` | `240 10% 7%` | Popover/dropdown backgrounds |
| `--popover-foreground` | `0 0% 98%` | Popover text |
| `--primary` | `0 0% 98%` | Primary buttons, emphasis |
| `--primary-foreground` | `240 5.9% 10%` | Text on primary |
| `--secondary` | `240 3.7% 15.9%` | Secondary surfaces |
| `--secondary-foreground` | `0 0% 98%` | Text on secondary |
| `--muted` | `240 3.7% 15.9%` | Muted backgrounds, scrollbar thumbs |
| `--muted-foreground` | `240 5% 64.9%` | Secondary text, labels, placeholders |
| `--accent` | `240 3.7% 15.9%` | Hover backgrounds |
| `--accent-foreground` | `0 0% 98%` | Text on accent |
| `--destructive` | `0 62.8% 30.6%` | Error/destructive actions |
| `--destructive-foreground` | `0 0% 98%` | Text on destructive |
| `--border` | `240 3.7% 20%` | Borders, dividers |
| `--input` | `240 3.7% 20%` | Input borders |
| `--ring` | `240 4.9% 83.9%` | Focus rings |
| `--norn` | `210 12% 49%` | Brand accent — links, branded elements |
| `--norn-foreground` | `0 0% 98%` | Text on norn accent |

#### Light Mode (defined but not currently active)

| Token | HSL Value |
|-------|-----------|
| `--background` | `0 0% 100%` |
| `--foreground` | `240 10% 3.9%` |
| `--primary` | `240 5.9% 10%` |
| `--primary-foreground` | `0 0% 98%` |
| `--secondary` | `240 4.8% 95.9%` |
| `--muted` | `240 4.8% 95.9%` |
| `--muted-foreground` | `240 3.8% 46.1%` |
| `--border` | `240 5.9% 90%` |
| `--ring` | `240 5.9% 10%` |
| `--norn` | `210 12% 49%` |

#### Chart Colors (Explorer)

| Token | HSL Value | Description |
|-------|-----------|-------------|
| `--chart-1` | `210 12% 49%` | Primary — norn blue |
| `--chart-2` | `210 20% 60%` | Lighter blue |
| `--chart-3` | `200 15% 42%` | Teal-blue |
| `--chart-4` | `170 12% 49%` | Cyan |
| `--chart-5` | `250 12% 49%` | Purple |

### Typography

**Font families:**
- Sans (body): `Inter` — CSS variable `--font-geist-sans`
- Mono (code, addresses, numbers): `JetBrains Mono` — CSS variable `--font-geist-mono`
- Both loaded from Google Fonts with `display: "swap"`

**Custom font sizes (Tailwind extension):**

| Class | Size | Line Height | Weight |
|-------|------|-------------|--------|
| `text-display` | 2.25rem (36px) | 2.5rem | 700 (bold) |
| `text-heading` | 1.5rem (24px) | 2rem | 600 (semibold) |
| `text-subheading` | 1.25rem (20px) | 1.75rem | 600 (semibold) |

Standard Tailwind sizes (`text-xs` through `text-xl`) used otherwise. Body text defaults to `text-sm` (14px) in the explorer, `text-base` (16px) on the website.

**Conventions:**
- Addresses, hashes, amounts, code: always `font-mono`
- Stat values: `tabular-nums` for aligned digits
- Table headers: `text-xs uppercase tracking-wider text-muted-foreground`
- Labels in stat cards: `text-xs text-muted-foreground uppercase tracking-wider`

### Border Radius

```
--radius: 0.5rem (8px)
```

| Tailwind class | Value |
|----------------|-------|
| `rounded-lg` | 8px (`var(--radius)`) |
| `rounded-md` | 6px (`calc(var(--radius) - 2px)`) |
| `rounded-sm` | 4px (`calc(var(--radius) - 4px)`) |

Badges use `rounded-full`.

### Spacing Conventions

| Context | Pattern |
|---------|---------|
| Page container | `max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6` |
| Section gaps (website) | `py-20 sm:py-24 lg:py-32` |
| Card header | `p-6 flex flex-col space-y-1.5` |
| Card content | `p-6 pt-0` |
| Stat card | `p-4` |
| Table header cells | `h-10 px-4 text-left` |
| Table data cells | `px-4 py-3` |
| Between sections | `space-y-6` or `gap-6` |
| Header/footer height | `h-14` (56px) |

### Animations

| Name | Keyframes | Duration | Usage |
|------|-----------|----------|-------|
| `fade-in` | opacity 0 → 1 | 0.4s ease-out | General entrance |
| `slide-in` | translateY(-4px), opacity 0 → 0 | 0.2s ease-out | Emerging content (explorer) |
| `slide-up` | translateY(8px), opacity 0 → 0 | 0.5s ease-out | Landing page sections (website) |
| `count-up` | translateY(4px), opacity 0 → 0 | 0.3s ease-out | Stat card values (explorer) |
| `pulse-dot` | opacity 1 → 0.4 → 1 | 2s ease-in-out infinite | Live status dot (explorer) |

Transitions: `transition-colors` for hover states, `transition-all duration-200 ease-in-out` for complex transitions.

### Component Patterns

#### Button (CVA variants)

| Variant | Classes |
|---------|---------|
| `default` | `bg-primary text-primary-foreground shadow hover:bg-primary/90` |
| `destructive` | `bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90` |
| `outline` | `border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground` |
| `secondary` | `bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80` |
| `ghost` | `hover:bg-accent hover:text-accent-foreground` |
| `link` | `text-primary underline-offset-4 hover:underline` |
| `norn` | `bg-norn text-norn-foreground shadow hover:bg-norn/90` |

Sizes: `default` (h-9 px-4), `sm` (h-8 px-3 text-xs), `lg` (h-10 px-8), `icon` (h-9 w-9).

The `norn` variant exists on the website but not the explorer (add it when needed).

#### Badge (CVA variants)

| Variant | Classes |
|---------|---------|
| `default` | `border-transparent bg-primary text-primary-foreground shadow` |
| `secondary` | `border-transparent bg-secondary text-secondary-foreground` |
| `destructive` | `border-transparent bg-destructive text-destructive-foreground shadow` |
| `outline` | `text-foreground` |
| `norn` | `border-transparent bg-norn/10 text-norn` |

Base: `rounded-full border px-2.5 py-0.5 text-xs font-semibold`.

#### Card

- Base: `rounded-lg border bg-card text-card-foreground shadow-sm`
- `CardHeader`: `flex flex-col space-y-1.5 p-6`
- `CardTitle`: `font-semibold leading-none tracking-tight`
- `CardDescription`: `text-sm text-muted-foreground`
- `CardContent`: `p-6 pt-0`
- `CardFooter`: `flex items-center p-6 pt-0`

#### StatCard (Explorer)

- Wraps Card with `p-4 overflow-hidden relative`
- Label: `text-xs text-muted-foreground uppercase tracking-wider`
- Value: `text-2xl font-semibold tabular-nums animate-count-up`
- Optional sparkline anchored to bottom
- Optional icon: `h-4 w-4 text-muted-foreground`

#### PageContainer

```tsx
<div className="mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
```

With optional `title` (text-heading), `description` (text-sm text-muted-foreground), and `action` slot.

#### EmptyState

- Centered column: `flex flex-col items-center justify-center py-12 text-center`
- Icon: `h-10 w-10 text-muted-foreground/50 mb-4`
- Title: `text-sm font-medium text-foreground`
- Description: `text-sm text-muted-foreground`

#### Data Tables

- Header row: `border-b border-border`
- Header cell: `h-10 px-4 text-left font-medium text-muted-foreground text-xs uppercase tracking-wider`
- Body row: `border-b border-border transition-colors hover:bg-muted/50`
- Clickable rows add `cursor-pointer`
- Mobile-hide columns: `hidden sm:table-cell`

#### Links

- Internal: `text-norn hover:underline`
- External: include `ExternalLink` icon from lucide-react
- MDX: `text-norn underline underline-offset-4 hover:text-norn/80`

### Status Indicators

| State | Color |
|-------|-------|
| Connected / success | `bg-green-500` |
| Connecting / warning | `bg-amber-500 animate-pulse` |
| Disconnected / error | `bg-zinc-500` |

### Responsive Breakpoints

Standard Tailwind: `sm:` (640px), `md:` (768px), `lg:` (1024px), `xl:` (1280px).

**Key patterns:**
- Navigation hidden on mobile (`md:hidden` for menu toggle, `hidden md:flex` for nav)
- Stats grid: `grid-cols-2 lg:grid-cols-5`
- Content grids: `sm:grid-cols-2`, `lg:grid-cols-2`
- Typography scales down: `text-4xl sm:text-5xl lg:text-6xl`

### Code Blocks (Website)

- Background: `hsl(240 10% 6%)`
- Syntax theme: `github-dark-default` (via rehype-pretty-code)
- Filter: `saturate(0.25) brightness(1.15)` — muted, desaturated
- Inline code: `bg-muted px-[0.4rem] py-[0.2rem] font-mono text-sm rounded`
- Copy button on hover with Check/Copy icon transition

### Custom Scrollbar

```css
.scrollbar-thin {
  scrollbar-width: thin;
  scrollbar-color: hsl(var(--muted)) transparent;
}
```

WebKit: 6px width, 3px border-radius, transparent track.

### Dark Mode Enforcement

Both web apps force dark mode:

```tsx
<ThemeProvider attribute="class" defaultTheme="dark" forcedTheme="dark" disableTransitionOnChange>
```

Never add light mode toggles. The `:root` (light) values exist for specification completeness but are inactive.

---

## Terminal UI (Wallet CLI)

### Color Scheme

Uses `console::Style` from the `console` crate.

| Style | Color | Usage |
|-------|-------|-------|
| `style_success()` | Green | Success checkmarks, confirmed status |
| `style_error()` | Red | Error messages |
| `style_warn()` | Yellow | Warnings, mnemonic box borders |
| `style_info()` | Cyan | Highlighted values, emphasis |
| `style_bold()` | Bold | Titles, section headers |
| `style_dim()` | Dim | Hints, timestamps, secondary info |
| Banner | Magenta + Bold | Startup ASCII art |

### Output Structure

All output uses **2-space indent** from the terminal edge:

```
  [icon] Message text
```

**Standard output pattern for commands:**
1. Leading blank line
2. Bold title (2-space indent)
3. Optional divider (32-char dim line: `────────────────────────────────`)
4. Content (table or key-value pairs)
5. Optional dim hints
6. Trailing blank line

### Symbols

| Symbol | Unicode | Usage |
|--------|---------|-------|
| `✓` | U+2713 | Success indicator |
| `●` | U+25CF | Status dot (active, registered) |
| `–` | U+2013 | Empty/none placeholder |
| `╔╗╚╝╠╣║` | Box-drawing | Mnemonic warning box |

No emoji. Strictly text-based aesthetic.

### Tables

Two table types via `comfy_table`:

**Data tables** (lists) — `UTF8_FULL` preset:
- Full box-drawing borders (┌─┬─┐ etc.)
- Dynamic column widths
- Used for: token lists, validators, transaction history, wallet list

**Info tables** (key-value) — `NOTHING` preset:
- No borders, clean aligned rows
- Used for: balance details, token metadata, node info, fees

**Cell styles:**
- `cell()` — plain text
- `cell_right()` — right-aligned (amounts, numbers)
- `cell_green()` — green (received, success)
- `cell_yellow()` — yellow (sent, warnings)
- `cell_cyan()` — cyan (emphasis, info)
- `cell_bold()` — bold
- `cell_dim()` — dim (secondary)

All tables printed with `print_table()` which applies 2-space indent to each line.

### Prompts

Uses `dialoguer` crate:
- `prompt_password(prompt)` — hidden input, single entry
- `prompt_new_password()` — hidden input with confirmation ("Passwords do not match")
- `confirm(prompt)` — yes/no, default false

### Formatted Output Helpers

- `print_success(msg)` — green `✓` + message
- `print_error(msg, hint)` — red `Error:` + message, optional dim hint below
- `print_divider()` — dim 32-char dashed line

### Amount Formatting

- Native NORN: 12 decimal places, comma-grouped whole part, trailing zeros stripped
- Custom tokens: variable decimals via `format_token_amount(amount, decimals)`
- Example: `1,234.56789012 NORN`

### Address Display

- Always `0x`-prefixed: `0x{hex}`
- Truncation: `0xabcde...23456` via `truncate_hex_string(s, half_len)`

---

## Cross-Surface Consistency Rules

1. **Norn Blue** (`hsl(210, 12%, 49%)`) is the single brand accent everywhere
2. **Monospace for data** — addresses, hashes, amounts, code always use mono font (JetBrains Mono on web, default mono in terminal)
3. **Dark backgrounds** — enforced on web, natural in terminal
4. **Minimal decoration** — no gradients, no illustrations, no emoji on web. Terminal uses only `✓`, `●`, `–`, and box-drawing characters
5. **Information-dense** — prioritize showing data over whitespace. Use tables not cards when displaying lists
6. **Consistent status colors** — green = success/received, yellow/amber = warning/sent, red = error/destructive, cyan = info
7. **Subtle animations** — 0.2–0.5s ease-out entrance transitions. Never bouncy or playful
8. **Typography hierarchy** — clear distinction between display, heading, subheading, body, and label sizes. Labels are always uppercase + tracking-wider + muted-foreground
