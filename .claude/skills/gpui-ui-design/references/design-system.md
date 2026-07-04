# Design System & Critical Patterns

The `ui` crate now carries a full **shadcn/Tailwind-parity component kit** on top of the
Zed base, plus a generic, rebrandable **design-token system** in `crates/ui/src/styles/`.
Read this before styling anything or building a new component. Living catalog =
`examples/ui_gallery` (run `cargo run -p ui_gallery`); every component also has a `preview()`.

## 1. Design tokens — the color/shape source of truth

Import via `use crate::styles::{palette, semantic};` (or through `ui::prelude::*` inside `ui`).

- **`semantic::*(cx)` → theme-driven NEUTRALS** (adapt light/dark automatically; read
  `cx.theme().colors()` under the hood). Use for every surface/border/text/hover:
  `background, surface, elevated_surface, card, popover, border, border_muted, border_focused,
  input_border, ring, text, text_muted, text_placeholder, hover_bg, active_bg, icon, icon_muted,
  secondary_bg, secondary_fg, muted_bg, accent_bg, accent_fg`.
- **`palette::role(step)` → mode-agnostic ACCENT/STATUS** ramps, `step ∈ 50..=950`:
  `neutral, primary, info, success, warning, danger`. Use for accents/status fills, e.g.
  `palette::primary(600)`, `palette::danger(600)`. (`info`≡primary ramp.)
- **`shadow::{Shadow, StyledShadow}`** → `el.shadow_level(Shadow::Sm|Base|Md|Lg|Xl)` (Tailwind
  box-shadow scale). Dropdowns/popovers = `Lg`, modals = `Xl`.
- **`focus_ring::{focus_ring_primary(content, focused), focus_ring_error(...)}`** → wraps an
  element in a true gapped focus ring (Tailwind `ring-2 ring-offset-2`), not a thick border.
- **`radius.rs`** → doc-reference mapping shadcn `--radius-*` → gpui's `rounded_sm`(4) `_md`(6)
  `_lg`(8) `_xl`(12) `_2xl`(16). Just call gpui's `.rounded_*()` directly.

**THE RULE:** neutrals → `semantic::*`; accents/status → `palette::*`; shadows → `shadow_level`;
focus → `focus_ring`. **Never hardcode `hsla(...)`/`0xRRGGBB`** in a component (except the
palette ramp definitions themselves + intentional fixed-dark surfaces like tooltip
`palette::neutral(900)`). `cx.theme().colors()` / `Color::Accent` still work but `semantic`/
`palette` are the preferred, rebrandable API.

## 2. Floating overlays (dropdown, select, popover, menu, hover-card, command)

Any popup MUST float above content — never a plain inline `.child(list)` (that pushes siblings
down). GPUI's portal-equivalent = **`deferred` + `anchored` + `occlude`**:

```rust
// measure the trigger once (store on the entity): trigger_bounds: Rc<Cell<Option<Bounds<Pixels>>>>
// ...in render, capture bounds via a canvas() child of the trigger row, then when open:
deferred(
    anchored()
        .snap_to_window_with_margin(px(8.))
        .position(bounds.origin + point(px(0.), bounds.size.height + px(4.)))
        .child(div().occlude().child(list)),
)
.with_priority(1)
```

Reference impls: `components/popover_menu.rs`, `context_menu.rs`, `select.rs`, `combobox.rs`,
`multi_select.rs`, `command.rs`, `navigation_menu.rs`, `date_picker.rs`.

## 3. Stateful component inside a parent view — store the Entity, don't recreate it

A stateful child (`Select`, `Combobox`, `Calendar`, `Menubar`, `Command`, `InputOtp`, …) is an
`Entity<T>`. **Create it ONCE and keep it as a field on the parent; render via `.clone()`.**
NEVER call `cx.new(|cx| Child::new(...))` inside a `render`/`preview` body — the parent
re-renders every frame, so a fresh entity is minted each time and all state (open menu, typed
text, selected day) is lost. This is the #1 gallery/composed-view bug.

```rust
struct MyView { picker: Entity<DatePicker> }
impl MyView {
    fn new(cx: &mut Context<Self>) -> Self { Self { picker: cx.new(|cx| DatePicker::new(cx)) } }
}
// render: .child(self.picker.clone())   // NOT cx.new(...)
```
Ctors needing `&mut Window` (e.g. `Menubar` via `ContextMenu::build`) can't run in `new()`:
build lazily-once with an `Option<Entity<_>>` + an `ensure_x(window, cx)` guard (see
`examples/ui_gallery/src/gallery_app.rs`).

## 4. Authoring a new component (match the kit)

- **Stateless** (most): `#[derive(IntoElement, RegisterComponent)]` struct + a role enum mapping
  to `palette::role(step)` + builder methods + `impl RenderOnce` + `impl Component { scope();
  description(); fn preview(window, cx) -> Option<AnyElement> }`. **Gold-standard template:
  `components/badge.rs`.** `RegisterComponent` puts it in the catalog.
- **Stateful**: an `Entity`/`View` (`impl Render`), constructed via `cx.new`, plus a
  `pub fn {name}_preview(window, cx) -> AnyElement` (Select/Combobox precedent) since a static
  `Component::preview` can't own live state. Expose getters (`selection()`, `value()`,
  `active_index()`) so tests/callers can read state without touching private fields.
- Keep files focused; no `.unwrap()`/`.expect()` on user input/text/index — propagate or guard.

## 5. Testing UI headlessly (the "computer-use" harness)

Use **`#[gpui::test]` + `TestAppContext`** (GPUI's mock `TestPlatform`, headless, runs on
`cargo test` worker threads cross-platform). Do NOT use `VisualTestAppContext`/real-Metal
windows in normal tests — they SIGABRT ("Rust cannot catch foreign exceptions") off the main
thread; reserve those for `#[ignore]` + `#[cfg(target_os="macos")]` manual screenshot smoke.

- Open: `cx.add_window(|_, cx| MyView::new(cx))` (or the gallery's `open_gallery(cx)` helper).
- Drive real events: `cx.simulate_click(pos, mods)`, `cx.simulate_input("text")`,
  `cx.simulate_keystrokes(...)`, `cx.simulate_event(ScrollWheelEvent{..})`, `run_until_parked()`.
- Target a control by adding a test-gated `debug_selector(|| "X".into())`
  (`#[cfg(any(test, feature="test-support"))]`, no-op in release) and clicking its
  `VisualTestContext::debug_bounds("X")`.
- Assert **real state** via `entity.read(cx).getter()` — never poke private fields or set a
  field and pretend it was a click.
- Run: `cargo test -p ui_gallery` (harness lives in `examples/ui_gallery/tests/visual_harness.rs`,
  the pattern reference). `crates/ui/src/components/context_menu.rs` shows the in-crate variant.

## 6. What's in the kit now (grouped — see `ui_gallery` page for each)

- **Elements** (Elements page): Button (`.variant(ButtonVariant::{Default,Destructive,Outline,
  Secondary,Ghost,Link})` + `.size(sm/default/lg/icon)`, plus legacy `.primary()/.danger()/
  .soft()`), Badge, Card, Avatar, Facepile, Chip, Divider, Separator, Skeleton, Spinner, Kbd,
  Toggle/ToggleGroup, AspectRatio, Label, Icon.
- **Forms** (Forms page): TextInput, Textarea, Select, Combobox, MultiSelect, SearchInput,
  Checkbox, Switch, RadioButton, Slider, InputGroup, SegmentedControl, FormField, ActionPanel,
  Form, FileInput, InputOtp.
- **Data** (Data page): Table, DataTable, List, DescriptionList, StatsCard, MediaObject,
  EmptyState, Feed.
- **Overlays** (Overlays page): Modal, AlertModal, Dialog, AlertDialog, Drawer/Sheet, Popover,
  HoverCard, Tooltip, DropdownMenu, ContextMenu, Menubar, Command, Sonner/ToastStack.
- **Navigation** (Navigation page): Navbar, Sidebar, VerticalNav, Tabs (`TabBarStyle::{Underline,
  Pills}`), Breadcrumb, Pagination, Stepper, NavigationMenu, Progress.
- **Layout** (Layout page): AppShell, PageHeading, SectionHeading, Container, Card, Resizable,
  Calendar, DatePicker, Carousel, Chart (hand-rolled `canvas()` Bar/Line/Area/Pie).

When a signature isn't here, read `crates/ui/src/components/<name>.rs` (vendored = matches what
compiles) and its `preview()`; or open the matching `examples/ui_gallery/src/pages/*.rs`.
</content>
