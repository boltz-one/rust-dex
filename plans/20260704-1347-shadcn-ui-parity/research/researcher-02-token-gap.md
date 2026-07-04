# Research: shadcn/ui token model vs GPUI UI kit gap matrix

Sources: https://ui.shadcn.com/docs/theming (WebFetch, fetched 2026-07-04) · `crates/ui/src/styles/{palette,semantic}.rs` · `crates/ui/src/components/*` · `crates/theme/src/styles/colors.rs` · prior audit `plans/20260703-0001-tailwind-app-ui-complete/research/researcher-02-codebase-audit.md`.

## Part A — shadcn token model

**CSS vars** (ui.shadcn.com/docs/theming): `--background/-foreground`, `--card/-foreground`, `--popover/-foreground`, `--primary/-foreground`, `--secondary/-foreground`, `--muted/-foreground`, `--accent/-foreground`, `--destructive` (+`-foreground` in current template [unverified — tool summary didn't confirm pair]), `--border`, `--input`, `--ring`, `--radius`. Plus `--sidebar*` (7 vars) and `--chart-1..5`. [source: WebFetch]

**Light/dark**: same var names, values swapped under a `.dark` class selector (class-based, not media-query-only) — attribute strategy typical of `next-themes`. [source: WebFetch]

**Color format**: current shadcn template ships **OKLCH** (`oklch(0.205 0 0)`), not raw HSL triplets — HSL was the *old* (pre-2024) shadcn convention (`222.2 84% 4.9%` consumed via `hsl(var(--x))`). User's HSL assumption is stale; cross-check if the local Tailwind config still targets HSL. [source: WebFetch; flagged since it contradicts the HSL premise in the task]

**Radius scale**: single `--radius` base drives derived tokens (`--radius-sm/-md/-lg/-xl`) via `calc()` multiples/offsets so one variable reskins all corners. Exact formula returned by the fetch (`*0.6`, `*0.8`) looks approximate — actual current shadcn CSS uses subtractive offsets (`calc(var(--radius) - 4px)` style in older versions). **Not independently verified — re-fetch raw `globals.css` before implementing.**

**Naming convention**: strict base+`-foreground` pairing per role (bg + guaranteed-contrast text/icon color), e.g. `primary`/`primary-foreground`. This is the core contract to preserve when mapping to Rust.

## Part B — codebase token map

`crates/ui/src/styles/palette.rs`: role ramps `neutral/primary/success/warning/danger/info` (50-950, Tailwind-sourced hex), mode-agnostic (no dark variant — same value both themes).

`crates/ui/src/styles/semantic.rs`: theme-aware neutral roles backed by `cx.theme().colors()` (`crates/theme/src/styles/colors.rs` `ThemeColors` struct): `background`, `surface` (`surface_background`), `elevated_surface` (`elevated_surface_background`), `border`, `border_muted` (`border_variant`), `border_focused`, `text`, `text_muted`, `text_placeholder`, `hover_bg` (`element_hover`), `active_bg` (`element_active`), `icon`, `icon_muted`.

### Token map: shadcn var → codebase

| shadcn var | codebase equivalent | status |
|---|---|---|
| `--background`/`-foreground` | `semantic::background`/`semantic::text` | ✅ have |
| `--card`/`-foreground` | `semantic::surface` + `semantic::text` | 🟡 no distinct "card" name, reuse surface — fine, shadcn's card is visually == generic surface in most themes |
| `--popover`/`-foreground` | `semantic::elevated_surface` + `semantic::text` | 🟡 same pattern, popover.rs component already exists |
| `--primary`/`-foreground` | `palette::primary(600)` + white/text | ✅ have (via `TintColor::Accent`) |
| `--secondary`/`-foreground` | **none** | ❌ missing role — no "always-visible muted solid" bg distinct from primary; closest is `palette::neutral(100..200)` used ad hoc |
| `--muted`/`-foreground` | `semantic::text_muted`, but no *background* muted role | 🟡 partial — text side covered, bg side missing |
| `--accent`/`-foreground` | `semantic::hover_bg`/`active_bg` (interaction-only, not a static bg role) | 🟡 different semantics: shadcn accent = a color role for hover/selected states AND standalone chips; codebase only has it as hover state |
| `--destructive`(-fg) | `palette::danger(600)` via `TintColor::Error` | ✅ have, different name |
| `--border` | `semantic::border` | ✅ have |
| `--input` | **none distinct** | ❌ missing — text_input.rs likely reuses `semantic::border`; no separate input-border role |
| `--ring` | `semantic::border_focused` (closest) + `styles/focus_ring.rs` helper exists | 🟡 have equivalent concept, different name/shape (ring vs border-focused) |
| `--radius` | **no radius scale file found** (not in `styles/` — only `units.rs`, no `radius.rs`) | ❌ missing centralized radius scale; components likely hardcode `.rounded_*()` Tailwind-style calls per-instance |
| sidebar-* | `sidebar.rs` component exists, unclear if uses dedicated palette | 🟡 unverified |
| chart-1..5 | **none** | ❌ missing, no chart component/tokens |

**Proposed additions** (research-only, not prescriptive code): add `semantic::muted_bg` (bg-only), `semantic::secondary_bg`/`secondary_fg` (new neutral-solid role, NOT reuse primary), `semantic::input_border` alias, `palette::radius(step)` or a `styles/radius.rs` with base+derived scale; keep `destructive`→`palette::danger`, `ring`→`semantic::border_focused` as documented aliases rather than renames (avoids churn, see below).

## Part B2 — component gap matrix (shadcn catalog vs `crates/ui/src/components/`)

✅ done / 🟡 partial-align / ❌ missing. Compiled from `ls components/` + prior audit; items marked [unverified] weren't opened this pass.

| shadcn | codebase file | status | note |
|---|---|---|---|
| Button | `button/` (button, button_like, icon_button, split_button, toggle_button, button_link, copy_button) | 🟡 | see Button deep-dive below |
| Badge | `badge.rs` | ✅ | Tailwind-aligned already (prior audit) |
| Alert | `alert.rs` | ✅ | |
| Avatar | `avatar.rs` | ✅ | |
| Card | `card.rs`/`container.rs` | ✅ | prior audit: "done" |
| Breadcrumb | `breadcrumb.rs` | ✅ | |
| Popover | `popover.rs`, `popover_menu.rs` | ✅ | |
| Dropdown Menu | `dropdown_menu.rs` | ✅ | |
| Context Menu | `context_menu.rs`/`right_click_menu.rs` | ✅ | |
| Dialog | `modal.rs` | 🟡 | generic modal, no dedicated AlertDialog subtype |
| Drawer/Sheet | `drawer.rs` | ✅ | shadcn treats Sheet≈Drawer, one file covers both |
| Sidebar | `sidebar.rs` | ✅ | |
| Tabs | `tab.rs`/`tab_bar.rs` | ✅ | |
| Tooltip | `tooltip.rs` | ✅ | |
| Pagination | `pagination.rs` | ✅ | |
| Data Table | `data_table/` | ✅ | shadcn has plain Table too — codebase only has data_table, no lightweight static Table 🟡 |
| Combobox | `combobox.rs` | ✅ | |
| Select | `select.rs`/`multi_select.rs` | ✅ | |
| Radio Group | `radio.rs` | ✅ | prior audit: done |
| Progress | `progress/` | ✅ | |
| Divider/Separator | `divider.rs` | ✅ | |
| Label | `label/` | ✅ | |
| Toggle | `toggle.rs` | ✅ | |
| Toggle Group | `toggle_button.rs`/`segmented_control.rs` | 🟡 | prior audit: partial |
| Accordion/Collapsible | `disclosure.rs` | 🟡 | one file for two shadcn concepts, unverified API split |
| Checkbox | `toggle.rs` (shared) [unverified] | 🟡 | prior audit: "restyle pending" |
| Switch | `toggle.rs` (shared) [unverified] | 🟡 | no dedicated switch.rs |
| Textarea | not in `ls` (likely `text_input.rs` multiline mode) [unverified] | 🟡 | |
| Input | `text_input.rs` | ✅ | |
| Command (palette) | ❌ missing | ❌ | prior audit confirms missing |
| Calendar/Date Picker | ❌ missing | ❌ | |
| Carousel | ❌ missing | ❌ | |
| Chart | ❌ missing | ❌ | no chart tokens either |
| Skeleton | ❌ missing | ❌ | |
| Slider | ❌ missing | ❌ | not in `ls` |
| Resizable | ❌ missing | ❌ | |
| Aspect Ratio | ❌ missing | ❌ | |
| Input OTP | ❌ missing | ❌ | |
| Menubar | ❌ missing | ❌ | navbar.rs ≠ menubar semantics |
| Hover Card | ❌ missing | 🟡 | tooltip.rs covers simple case only |
| Scroll Area | `scrollbar.rs` | 🟡 | raw primitive, not a styled wrapper region |
| Toast/Sonner | `notification/` [unverified name mapping] | 🟡 | |

### Button deep-dive (explicit ask)

`ButtonStyle` (button_like.rs:131): `Filled`, `Tinted(TintColor)` where `TintColor` = `Accent/Error/Warning/Success`, `Outlined`, `OutlinedGhost`, `OutlinedCustom(Hsla)`, `Subtle` (default), `Transparent`.
`ButtonSize` (button_like.rs:472): `Large/Medium/Default/Compact/None` — rems 32/28/22/18. No `icon` size variant; icon-only buttons are a **separate component** `IconButton`, not a Button size (architectural difference, not a bug — KISS via composition vs shadcn's single-component+size="icon").

Mapping vs shadcn `default/destructive/outline/secondary/ghost/link` + `default/sm/lg/icon`:

- `default` → `Filled` + `Tinted(Accent)` / convenience `.primary()` — ✅ roughly covered
- `destructive` → `Tinted(Error)` / `.danger()` — ✅ covered, different name
- `outline` → `Outlined` — ✅ covered
- `secondary` → ❌ **no equivalent** — `Subtle` is transparent-until-hover, not shadcn's always-visible muted-solid `secondary`. Needs new variant or reuse of missing `secondary` token above.
- `ghost` → `Transparent` (close) or `OutlinedGhost` (has a border on some state, less exact) — 🟡
- `link` → `button_link.rs` (`ButtonLink`) exists as a **separate component**, not a ButtonStyle variant — ✅ covered but architecturally split
- sizes `sm/default/lg` → `Compact/Default/Medium|Large` roughly maps — 🟡 naming differs, no 1:1
- `icon` size → `IconButton` component — 🟡 architectural split, not a gap per se

## Churn estimate (grep, `crates/ui` + workspace)

- `ButtonStyle::` usages: **102**
- `.primary()` convenience calls: **18**
- `.danger()` convenience calls: **6**
- `.soft()` convenience calls: **4** (semantics unverified — likely Badge-style soft bg, not confirmed on Button itself within budget)

Renaming `ButtonStyle` variants to match shadcn names (`Filled→Default`, `Tinted(Error)→Destructive`, etc.) touches **~130 call sites** directly, plus any component/story files not grepped. High-churn if literal renames chosen. Lower-risk path: keep existing enum, add shadcn-named builder aliases (`.default_style()`, `.destructive()`, `.secondary()`) — zero churn, additive only, consistent with DRY/YAGNI (don't rename what already has 100+ callers unless the plan explicitly needs the shadcn vocabulary as the public API).

## Open questions

1. Exact current shadcn `--radius-*` calc formula — WebFetch summary looked imprecise vs known shadcn CSS; re-verify raw `globals.css` from a shadcn `init` output before designing `radius.rs`.
2. Does shadcn's current template actually ship `--destructive-foreground`? Not confirmed by fetch.
3. Are `--input` and `--ring` semantically distinct enough from `--border`/`border_focused` in this codebase's actual rendering, or already visually identical (i.e., is the "gap" purely nominal)?
4. Do `Checkbox`/`Switch`/`Textarea` have dedicated source files not caught by `ls components/` (e.g. nested under `label/`, `list/`, or nested subfolders not listed), or do they genuinely share `toggle.rs`/`text_input.rs`? Needs a direct file read.
5. `.soft()` builder — which file defines it and what `ButtonStyle`/color it maps to? Not located within tool budget.
6. Does `sidebar.rs` already consume a dedicated palette akin to shadcn's `--sidebar-*` set, or generic `semantic::*`?
