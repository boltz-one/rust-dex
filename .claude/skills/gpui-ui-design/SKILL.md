---
name: gpui-ui-design
description: Build, compose, and style GPUI UI for the Boltz/rust-destop desktop app — draw screens, design layouts, and use the Zed-derived `ui` component crate (Button, Label, Icon, List, Modal, Tooltip, ContextMenu, etc.) correctly. Use this skill whenever the user asks to build, draw, design, prototype, or assemble any UI, screen, panel, dialog, form, toolbar, or component using GPUI / `ui::prelude` / `div()` / `h_flex` / `v_flex`, or says things like "vẽ UI", "thiết kế giao diện", "tạo screen/view", "dùng component", "GPUI button/label/modal", even when they don't name the framework explicitly. This project IS a vendored GPUI stack (crates/gpui, crates/ui, crates/theme, crates/icons), so this skill applies to all UI work here.
metadata:
  origin: agent-learned
---

# GPUI UI Design & Components

This project (`rust-destop` / "Boltz") is a vendored copy of Zed's UI stack. The UI lives in:
- `crates/gpui` — the framework (elements, `div()`, `Styled`, entities, actions, `Window`/`App`/`Context`).
- `crates/ui` — the component crate (`Button`, `Label`, `Icon`, `List`, `Modal`, `Tooltip`, `ContextMenu`, …). Re-exported via `ui::prelude::*`.
- `crates/theme` — colors, typography, `ActiveTheme` (`cx.theme()`).
- `crates/icons` — `IconName` (~290 SVG icons).

This skill teaches how to think in GPUI and use these components idiomatically. The reference files hold the verbatim API; this file holds the mental model.

## How to think in GPUI (read first if new)

GPUI is a **retained, flexbox-based, entity-driven** UI framework — not the DOM, not React, not immediate-mode. Three mental shifts matter:

1. **Everything is a `Div` tree styled like Tailwind.** `div().flex().gap_2().p_4().child(...)` is the unit of layout. Style methods are Tailwind-ish (`p_2`, `gap_4`, `flex_row`, `items_center`, `rounded_sm`, `bg(...)`, `text_xs`). Numeric suffixes are a rem ramp where `1` ≈ 4px.
2. **Stateful things are `Entity<T>` ("views").** A piece of state `T` that can render becomes `Entity<T>` via `cx.new(|cx| T { ... })`. `T: Render` gives it `render(&mut self, window, cx) -> impl IntoElement`. To mutate, `entity.update(cx, |t, cx| { ...; cx.notify() })` — `cx.notify()` triggers a rerender.
3. **Events close over the entity via `cx.listener`.** Handlers like `on_click`/`on_action` want `Fn(&Event, &mut Window, &mut App)`. To reach `&mut T` inside, wrap with `cx.listener(|this, event, window, cx| { ... })` or `cx.listener(Self::method)`. A plain `div()` must get `.id("...")` before it can take `on_click`/`on_hover`/`on_action` (it becomes stateful).

**Why this matters:** forgetting `.id()` is the #1 cause of "method not found on `Div`" errors; forgetting `cx.notify()` is the #1 cause of "my UI doesn't update".

For the full mental model + a copy-paste view template, read **`references/views-and-state.md`**.

## The standard workflow

When asked to build/draw/design a UI, work top-down:

1. **Decide the view.** What state does it hold? What actions does it handle? Sketch the struct fields (state + `_subscriptions: Vec<Subscription>` + optional `FocusHandle`).
2. **Pick layout primitives.** Start from `v_flex()` / `h_flex()` (flex + direction, **no default gap** — you add `.gap_N()`). For grouped control rows use `h_group()`/`v_group()` (flex + a fixed gap by size). See **`references/layout-styling.md`**.
3. **Reach for a component before raw `div()`.** The `ui` crate already has Button, Label, Icon, IconButton, Tooltip, List/ListItem, Modal, ContextMenu, DropdownMenu, Disclosure, Toggle/Switch/Checkbox, Banner, Callout, ProgressBar, Avatar, etc. Using them gives you theme-correct colors, a11y, focus, and hover for free. See **`references/components.md`** for the verbatim constructor + builder methods of every one.
4. **Wire interactions** with `on_click`/`on_action` + `cx.listener`, mutate state, `cx.notify()`.
5. **Use theme, not raw colors.** `cx.theme().colors().text`, `Color::Accent`, `Color::Error`, `Severity::Warning`. `Color` carries semantic meaning across themes — prefer it over `Hsla` literals. See **`references/components.md`** § Colors & Severity.
6. **Build & check.** `cargo check -p <crate>` (the project uses `./script/clippy` for clippy, mirroring Zed). Never `unwrap()`; propagate with `?` or `.log_err()`.

## The one import you need

```rust
use ui::prelude::*;
```

This brings in `div`, `h_flex`, `v_flex`, `Button`, `Label`, `Icon`, `IconName`, `Color`, `Tooltip`, `px`, `rems`, `ActiveTheme`, and all the style methods. Add `use ui::TintColor;` / `use ui::Tooltip;` etc. when a non-prelude symbol is needed.

## Minimum viable screen

```rust
use ui::prelude::*;

struct MyView { count: usize }

impl Render for MyView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_4()
            .child(Label::new(format!("Count: {}", self.count)).size(LabelSize::Large))
            .child(
                Button::new("inc", "Increment")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.count += 1;
                        cx.notify();
                    })),
            )
    }
}
```

This compiles in this project. To see how a view is **bootstrapped into a window** (app init, theme init, `cx.open_window`), read **`references/app-bootstrap.md`** — it is the verbatim pattern from `crates/app/src/main.rs`.

## Reference files (read as needed)

- **`references/components.md`** — The catalog. Constructor + key builders + one idiomatic example for every `ui` component (buttons, labels, icons, lists, modals, menus, toggles, indicators, progress, banner/callout, avatar/facepile, tooltip, keybinding). **Read this before writing any component code** — it has the exact signatures.
- **`references/layout-styling.md`** — `div()`/`h_flex`/`v_flex`/`h_group`/`v_group`, the Tailwind-style method families (padding/margin/size/gap/flex/align/justify/border/rounded/overflow/text), the rem ramp, and conditional builders (`.when`/`.when_some`/`.when_else`).
- **`references/views-and-state.md`** — `Entity<T>`, `Render`, `cx.new`/`update`/`notify`, `cx.listener`, `EventEmitter`/`subscribe`, `Focusable`, actions (`actions!` + `on_action`), and a full real-world view template.
- **`references/app-bootstrap.md`** — How `main.rs` initializes `gpui_platform::application()`, theme, and opens a window. Copy-paste ready.
- **`references/idioms.md`** — Common pitfalls (`.id()` before handlers, `cx.notify()`, no `unwrap`, `SharedString`, conditional children) and "how do I do X" recipes.

## When the API isn't here

This skill catalogs the common path. If a component or method isn't covered, the source of truth is `crates/ui/src/components/<name>.rs` and `crates/gpui/src/` in THIS repo (it is vendored, so read the local copy — it matches what compiles here). Each component file has a `preview()` fn showing gold-standard usage; grep `fn preview` to find examples.
