# Idioms & pitfalls — the things that bite

A cheat-sheet of the GPUI mistakes that cause most compile errors and "why doesn't this work" moments. Read when something won't build or won't update.

## Compile-time pitfalls

### 1. `no method named on_click/on_hover/on_action on Div`
`div()` is **stateless** — interaction handlers need a **stateful** element. Add `.id("...")`:
```rust
// ❌
div().on_click(|_, _, _| {})

// ✅
div().id("clickable").on_click(|_, _, _| {})
```
This applies to raw `div()`. Components (`Button`, `IconButton`, `ListItem`, …) are already interactive — don't add `.id()` to them, just call `.on_click(...)`.

### 2. Using the outer `cx` inside an `update`/`listener` closure
The closure receives its own `cx`; using the outer one double-borrows and panics at runtime.
```rust
// ❌ panics: re-entrant borrow
entity.update(cx, |e, cx2| { e.do_thing(cx); })   // used outer `cx` instead of `cx2`

// ✅
entity.update(cx, |e, cx| { e.do_thing(cx); })    // use the inner cx
```
Inside `cx.listener(|this, event, window, cx| ...)`, use that `cx`.

### 3. `unwrap()` / `expect()` / indexing that can panic
Repo rule (from `CLAUDE.md`): never `unwrap()` on fallible ops — propagate with `?`, ignore-and-log with `.log_err()`, or `match`/`if let`. Index with care; prefer `.get(i)`.

### 4. Missing `use ui::prelude::*`
Without it, `div`, `h_flex`, `Button`, `Label`, `Color`, `px`, `rems`, `ActiveTheme`, and all style methods are unresolved. Some symbols live outside the prelude — add them explicitly: `use ui::Tooltip;`, `use ui::TintColor;`, `use ui::ElevationIndex;`.

### 5. `IconName::Search` / `Save` / `Sync` / `Moon` — these don't exist
`IconName` has ~265 variants but many "obvious" names are absent and will fail `cargo check` with E0599. Common traps: `Search`→`MagnifyingGlass`, `Save`→(none, use `Download`/`Check`), `Sync`/`Refresh`→`RotateCcw`, `Moon`/`Sun`→(none). **Always grep before using:** `grep -E "^\s+(NameHere)," crates/icons/src/icons.rs`. Full gotcha list in `references/components.md` § IconName.

### 6. `&str` where `SharedString` / `impl Into<SharedString>` is wanted
Most component constructors take `impl Into<SharedString>` — a `&str`, `String`, or `SharedString` all work. But if you build a string conditionally, hand it a `String` or `.into()`. `SharedString` is either `&'static str` or `Arc<str>` — cheap to clone, so clone freely.

## Runtime pitfalls

### 7. UI doesn't update after state change → missing `cx.notify()`
After mutating view state in a handler/`update`, call `cx.notify()`:
```rust
.on_click(cx.listener(|this, _, _, cx| {
    this.count += 1;
    cx.notify();   // ← without this, render() is never re-called
}))
```
Not needed inside `render()` itself (already rendering).

### 8. Dropped subscription / task → callback silently stops
`Subscription` and `Task` cancel when dropped. Store them:
```rust
struct View { _subscriptions: Vec<Subscription>, _task: Option<Task<()>> }
```
The `_` prefix documents "kept for its drop side-effect." A `cx.observe(...).detach()` is fine to NOT store (it self-keeps) — but a `cx.subscribe(...)` should usually be stored so you control its lifetime.

### 9. Forgot the gap → children touching
`h_flex()`/`v_flex()` set **no gap**. Always add `.gap_N()` (or use `h_group()`/`v_group()` which bake one in).
```rust
// ❌ cramped
h_flex().child(a).child(b)

// ✅
h_flex().gap_2().child(a).child(b)
```

### 10. Text overflowing its flex row
A flex child won't shrink below content width by default. To truncate:
```rust
h_flex().min_w_0().overflow_hidden()        // parent lets it shrink
    .child(Label::new(long_text).truncate()) // child truncates with …
```
Or `Label::new(t).truncate_start()` / `.truncate_middle()` / `.line_clamp(n)`.

### 11. Re-entrant update panic
Don't call `entity.update(cx, ...)` on an entity that's already mid-`update`. If you need to react to a change inside an update, defer with `cx.notify()` + an `observe`, or schedule on the next tick.

## "How do I…" recipes

### Add a tooltip to a button
```rust
Button::new("x", "Delete").tooltip(Tooltip::text("Delete the selected item"))
IconButton::new("y", IconName::Search).tooltip(move |_, cx| Tooltip::for_action("Search", &Search, cx))
```

### Make a primary + secondary button row (modal footer)
```rust
ModalFooter::new().end_slot(
    h_flex().gap_2()
        .child(Button::new("cancel", "Cancel").style(ButtonStyle::Subtle))
        .child(Button::new("ok", "Confirm").style(ButtonStyle::Filled))
)
```

### Spacer (push siblings apart)
```rust
h_flex().w_full()
    .child(left)
    .child(div().flex_1())   // grows, pushes `right` to the far edge
    .child(right)
```
Or use `.justify_between()` on the row.

### Conditionally render a child
```rust
div().when_some(self.error, |this, err| this.child(Label::new(err).color(Color::Error)))
// or
div().when(self.show_badge, |this| this.child(CountBadge::new(3)))
```
Prefer `.when`/`.when_some` over `if let`/`match` building two trees — it keeps one builder chain and one return type.

### Render a `Vec` of items
```rust
v_flex().gap_1().children(self.items.iter().map(|item| {
    ListItem::new(item.id).child(Label::new(item.label.clone()))
}))
```

### Dispatch an action from a click
```rust
.on_click(cx.listener(|_this, _, window, cx| {
    window.dispatch_action(Box::new(MyAction), cx);
}))
```

### Toggle a view open/closed
```rust
Disclosure::new("section", self.open)
    .on_toggle_expanded(cx.listener(|this, _, _, cx| { this.open = !this.open; cx.notify(); }))
```

### Build a context menu and show it on right-click
```rust
let menu = ContextMenu::build(window, cx, |this, _, _| {
    this.entry("Open", None, |_, _| {}).entry("Delete", None, |_, _| {})
});
right_click_menu::<ContextMenu>("ctx")
    .menu(move |_, _| Some(menu.clone()))
    .trigger(move |_active, _, _| Label::new("right-click me"))
```

### Use design tokens instead of hardcoded HSLA
```rust
// neutrals (surface/border/text/hover) — theme-driven, dark+light for free:
div().bg(semantic::surface(cx)).border_color(semantic::border(cx))
// accents/status — mode-agnostic ramps:
div().bg(palette::primary(600))
// text still goes through the semantic Color enum:
Label::new("Error").color(Color::Error)
```
Never write `hsla(...)`/`0xRRGGBB` literals or reach for raw `cx.theme().colors().*` in new code —
`semantic`/`palette` are the current API (`references/design-system.md` §1). `Color::Custom(hsla)`
and direct `cx.theme()` calls still compile (older components use them) but are legacy, not the
pattern to copy.

## Source of truth reminder
When a method/component isn't in this skill's references, read the local vendored source — it matches what compiles here:
- `crates/ui/src/components/<name>.rs` — each has a `preview()` fn = gold-standard example. `grep -rn "fn preview" crates/ui/src/components/`.
- `crates/gpui/src/styled.rs` + `crates/gpui_macros/src/styles.rs` — the full style-method families and rem ramp.
- `crates/icons/src/icons.rs` — every `IconName` variant, exact spelling.
