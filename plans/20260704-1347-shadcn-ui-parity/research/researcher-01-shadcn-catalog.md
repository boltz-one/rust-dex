# shadcn/ui Full Catalog — GPUI Port Baseline

Source: https://ui.shadcn.com/docs/components (live index, fetched 2026-07-04) + per-component docs (button/dialog/command/chart) + training knowledge (Radix primitives, cutoff Jan-2026, cross-checked against fetched pages — no conflicts found). GPUI-feasibility judged from this repo's `crates/ui/*.rs` (already-existing native equivalents listed) [file:crates/ui].

Legend: ✅ native feasible · 🟡 complex (needs custom render/layout work) · ⚠️ deferred (low desktop-native value or heavy dep).

## Elements

| Component | Variants | Sizes | States | Anatomy | GPUI | Note |
|---|---|---|---|---|---|---|
| Button | default/destructive/outline/secondary/ghost/link | default/xs/sm/lg/icon(+icon-xs/sm/lg) | hover/focus/disabled/loading | Button, ButtonGroup, Spinner-slot | ✅ | `crates/ui/button.rs` exists |
| Button Group | — | — | — | wrapper only | ✅ | `button.rs`/`group.rs` |
| Badge | default/secondary/destructive/outline | default/sm | — | single | ✅ | `badge.rs` |
| Avatar | — | sm/default/lg | fallback/loading | Root/Image/Fallback | ✅ | `avatar.rs` |
| Kbd | — | — | — | single/group | ✅ | new component, trivial (styled text) |
| Spinner | — | sm/default/lg | — | single | ✅ | new component, trivial |
| Separator | horizontal/vertical | — | — | single | ✅ | `divider.rs` |
| Skeleton | — | — | pulse anim | single div | ✅ | missing in crate — trivial to add (animated rect) |
| Aspect Ratio | — | ratio prop | — | wrapper | ✅ | trivial layout constraint, missing but easy |
| Progress | — | — | value/indeterminate | Root/Indicator | ✅ | `progress.rs` |
| Toggle | default/outline | sm/default/lg | on/off/disabled | single | ✅ | `toggle.rs` |
| Toggle Group | single/multiple | sm/default/lg | selected item | Root/Item | ✅ | build on `toggle.rs`+`segmented_control.rs` |
| Typography | headings/p/blockquote/code/list | — | — | n/a (styles) | ✅ | `typography.rs` |
| Empty | — | — | — | Header/Media/Title/Description/Content | ✅ | `empty_state.rs` |
| Item | — | — | — | Root/Media/Content/Actions | ✅ | list-row primitive, close to `list.rs` |
| Field | — | — | error/disabled | Root/Label/Control/Description/Error | ✅ | `form_field.rs` |

## Forms

| Component | Variants | Sizes | States | Anatomy | GPUI | Note |
|---|---|---|---|---|---|---|
| Input | — | sm/default/lg | focus/disabled/invalid | single | ✅ | `text_input.rs` |
| Input Group | — | — | — | Group/Addon/Input | ✅ | `input_group.rs` exists |
| Textarea | — | — | focus/disabled | single | ✅ | extend `text_input.rs` (multiline) |
| Label | — | — | disabled(peer) | single | ✅ | `label.rs` |
| Checkbox | — | sm/default | checked/indeterminate/disabled | single | ✅ | missing dedicated file — add via `toggle.rs` pattern |
| Radio Group | — | — | checked/disabled | Root/Item | ✅ | `radio.rs` |
| Switch | — | sm/default | checked/disabled | single | ✅ | build on `toggle.rs` |
| Select | — | sm/default | open/disabled | Trigger/Content/Item/Group/Label | ✅ | `select.rs` exists |
| Native Select | — | — | — | wraps OS `<select>` | ⚠️ | web-only concept, N/A desktop — skip |
| Combobox | — | — | open/typing/selected | Popover+Command composition | 🟡 | `combobox.rs` exists; needs fuzzy filter |
| Slider | — | — | dragging/disabled/range | Root/Track/Range/Thumb | 🟡 | missing; needs pointer-drag geometry math, no file yet |
| Input OTP | — | — | filled/focused slot | Group/Slot/Separator | 🟡 | missing; per-slot focus mgmt + paste-split logic, non-trivial but doable |
| Form (react-hook-form) | n/a | n/a | valid/invalid/touched | n/a | 🟡 | GPUI has no RHF; port as layout+validation-state struct only, no schema-lib parity |

## Overlays

| Component | Variants | Sizes | States | Anatomy | GPUI | Note |
|---|---|---|---|---|---|---|
| Dialog | — | — | open/closed | Root/Trigger/Content/Overlay/Header/Title/Description/Footer/Close | ✅ | `modal.rs` exists |
| Alert Dialog | — | — | open/closed | same as Dialog + Action/Cancel | ✅ | thin variant of `modal.rs` |
| Sheet | side: top/right/bottom/left | — | open/closed | same anatomy as Dialog | ✅ | `drawer.rs` covers this |
| Drawer (vaul) | — | — | open/dragging(mobile) | Root/Trigger/Content/Header/Footer | ✅ | `drawer.rs`; drag-to-dismiss gesture 🟡 sub-part |
| Popover | — | — | open/closed | Root/Trigger/Content | ✅ | `popover.rs` |
| Hover Card | — | — | open(on hover)/closed | Root/Trigger/Content | ✅ | `tooltip.rs`/`popover.rs` compose easily |
| Tooltip | — | — | open/closed | Provider/Root/Trigger/Content | ✅ | `tooltip.rs` |
| Dropdown Menu | — | — | open/checked(items)/disabled | Root/Trigger/Content/Item/CheckboxItem/RadioItem/Sub/Separator/Label | ✅ | `dropdown_menu.rs` |
| Context Menu | — | — | same as Dropdown | same anatomy | ✅ | `context_menu.rs`, `right_click_menu.rs` |
| Menubar | — | — | open/active-menu | Root/Menu/Trigger/Content/Item/Sub | 🟡 | no file; combine `navbar.rs`+`dropdown_menu.rs`, needs cross-menu keyboard nav |
| Sonner (toast) | success/error/info/loading/default | — | entering/visible/exiting | stacked-toast queue | 🟡 | `notification.rs` exists but is single-toast; needs stack/queue + auto-dismiss timers |
| Toast (Radix, deprecated by shadcn in favor of Sonner) | — | — | open/closed | Provider/Root/Title/Description/Action | ✅ | superseded — skip, use Sonner pattern only |

## Data & Navigation

| Component | Variants | Sizes | States | Anatomy | GPUI | Note |
|---|---|---|---|---|---|---|
| Tabs | — | — | active/disabled | Root/List/Trigger/Content | ✅ | `tab.rs`/`tab_bar.rs` |
| Breadcrumb | — | — | — | Root/List/Item/Link/Separator/Ellipsis | ✅ | `breadcrumb.rs` |
| Pagination | — | — | active/disabled | Root/Content/Item/Link/Prev/Next/Ellipsis | ✅ | `pagination.rs` |
| Navigation Menu | — | — | open submenu | Root/List/Item/Trigger/Content/Link/Viewport/Indicator | 🟡 | no file; needs animated viewport + hover-intent timers, closest to `navbar.rs` |
| Accordion | single/multiple | — | open/closed/disabled | Root/Item/Trigger/Content | ✅ | `disclosure.rs` |
| Collapsible | — | — | open/closed | Root/Trigger/Content | ✅ | `disclosure.rs` |
| Table | — | — | — | Root/Header/Body/Footer/Row/Head/Cell/Caption | ✅ | `data_table.rs` |
| Data Table (TanStack) | — | — | sorted/filtered/selected | built on Table | 🟡 | `data_table.rs` exists but TanStack sort/filter/pagination logic must be reimplemented in Rust |
| Scroll Area | — | — | scrolling/hover-thumb | Root/Viewport/Scrollbar/Thumb/Corner | ✅ | `scrollbar.rs` — GPUI has native scroll already, mostly styling |
| Sidebar | — | collapsed/expanded | — | Provider/Sidebar/Header/Content/Footer/Group/Menu/Trigger/Rail | ✅ | `sidebar.rs` exists, most complex composite but already ported |

## Layout

| Component | Variants | Sizes | States | Anatomy | GPUI | Note |
|---|---|---|---|---|---|---|
| Card | — | — | — | Root/Header/Title/Description/Content/Footer/Action | ✅ | `card.rs` |
| Direction (RTL/LTR provider) | ltr/rtl | — | — | Provider | 🟡 | GPUI text/layout RTL support must be checked; likely partial |

## Advanced / Heavy — go/no-go needed

| Component | What it wraps | GPUI | Note |
|---|---|---|---|
| Chart | Recharts (SVG/DOM charts, Bar/Line/Area/Pie/Radar) [WebFetch:ui.shadcn.com/docs/components/chart] | ⚠️ deferred | No SVG/canvas charting lib in GPUI stack; needs custom draw-primitives (paths/arcs) or embed a Rust plotting crate (plotters/charming) rendered to image. High effort, own sub-project. |
| Calendar (react-day-picker) | — | 🟡 complex | Date-grid layout + locale/i18n month math is doable natively but nontrivial (~1-2wk); no existing file. |
| Date Picker | Calendar + Popover composition | 🟡 complex | Depends on Calendar above; straightforward once Calendar exists. |
| Carousel (embla-carousel) | JS drag/snap physics lib | 🟡 complex | Needs custom drag-inertia + snap-point animation in GPUI; no embla equivalent in Rust ecosystem — must hand-roll. |
| Resizable (react-resizable-panels) | — | ✅ feasible | GPUI already does flex layout + can track pointer-drag deltas on divider; simpler than web since no DOM reflow cost. |
| Command (cmdk) | cmdk fuzzy-search list [WebFetch:ui.shadcn.com/docs/components/command] | 🟡 complex | Needs: text input + live fuzzy filter + keyboard up/down/enter + grouped virtualization. Buildable on `combobox.rs`+`list.rs`, no fuzzy-match crate wired yet (consider `nucleo`/`fuzzy-matcher`). |
| Sonner (toast stack) | sonner JS lib | 🟡 complex | See Overlays row above — stack/queue/timer logic, no existing file for multi-toast. |
| Input OTP | input-otp JS lib | 🟡 complex | See Forms row — per-slot state machine, moderate effort. |

## Open Questions
- Exact current variant/size list for Select, Checkbox, Slider not doc-fetched (relied on training knowledge; risk of drift if shadcn changed props recently — verify against `npx shadcn@latest add <x>` source before implementing).
- "New Components" section (Attachment/Bubble/Marker/Message/Message Scroller) appears AI-chat-specific and out of scope for a general UI-kit port — confirm with user if needed.
- No fetch done for Calendar/Carousel/Slider/InputOTP doc pages directly (tool-call budget spent on index+Button+Dialog+Command+Chart) — feasibility notes for those are training-knowledge + inference from GPUI primitives, not doc-verified.
- GPUI RTL/bidi text support unverified — affects "Direction" component feasibility.

## Trade-offs
- Charting: build custom (full control, GPUI-native, high effort) vs. render an image via a Rust plotting crate off-thread (fast to ship, less interactive/animatable).
- Command palette: reuse existing `combobox.rs` (fast, may lack fuzzy-rank quality) vs. new dedicated module with a fuzzy crate (better UX, more code).
- Toast/Sonner: extend `notification.rs` to a queue (less new code, risk of leaky abstraction) vs. new dedicated `toast_stack.rs` (clean, more files).
