# Views, state, and interactions — `Entity<T>`, `Render`, `cx.listener`, actions

Source of truth: `crates/gpui/src/app.rs`, `crates/gpui/src/app/context.rs`, and the GPUI section of the repo's `CLAUDE.md` / `zed/CLAUDE.md`.

## Context types (the `cx` argument)

- `App` — root context; global state, read/update entities. Functions taking `&App` also accept `&Context<T>` (it derefs to `App`).
- `Context<T>` — given when updating `Entity<T>`. Use the **inner** `cx` inside closures.
- `AsyncApp` / `AsyncWindowContext` — from `cx.spawn` / `cx.spawn_in`; held across `.await`.
- `Window` — the window; passed as `window` (before `cx`). Focus, dispatch, drawing, input.

Convention: callback args are `(event, window, cx)` or, for listeners, `(this, event, window, cx)`. `window` comes before `cx`.

## Entities — `Entity<T>`

An `Entity<T>` is a handle to heap state `T`. When `T: Render`, the entity is a "view".

```rust
struct Counter { count: usize }

// Create
let counter: Entity<Counter> = cx.new(|cx| Counter { count: 0 });

// Read
let n: &usize = counter.read(cx).count;
let n = counter.read_with(cx, |c: &Counter, _cx| c.count);

// Mutate (the inner cx must be used)
counter.update(cx, |c: &mut Counter, cx: &mut Context<Counter>| {
    c.count += 1;
    cx.notify();   // <-- triggers rerender. Without it, the UI will NOT update.
});

// Mutate + window
counter.update_in(cx, |c: &mut Counter, window: &mut Window, cx: &mut Context<Counter>| { /* ... */ });

// Weak handle (avoid reference cycles)
let weak: WeakEntity<Counter> = counter.downgrade();
weak.update(cx, |c, cx| { /* ... */ }).ok();  // Result — fails if dropped
```

**Inside an `update` closure, always use the `cx` the closure received**, not the outer `cx` — using the outer one double-borrows and panics. Never call `update` on an entity while it is already being updated (re-entrant update panics).

## `Render` — turning state into an element tree

```rust
struct Greeting { name: SharedString }

impl Render for Greeting {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Label::new(format!("Hello, {}!", self.name))
    }
}
```
`render` takes `&mut self` + `window` + `cx` and returns `impl IntoElement` (a `Div`, a component, anything `IntoElement`). It is called whenever the entity is notified.

## `RenderOnce` vs `Render`

- `Render` — for stateful views (`Entity<T>`). `render(&mut self, window, cx: &mut Context<Self>)`.
- `RenderOnce` — for components constructed just to be turned into elements (no long-lived entity). `render(self, window, cx: &mut App)` (takes ownership). Add `#[derive(IntoElement)]` to use the type directly as a child. This is how `Button`, `Label`, `Icon`, etc. work.

## `cx.listener` — reach entity state from an event handler

Event handlers (`on_click`, `on_hover`, `on_action`, …) want `Fn(&Event, &mut Window, &mut App)`. To mutate the view inside, wrap with `cx.listener`:

```rust
// closure form
.on_click(cx.listener(|this: &mut MyView, _event, window, cx| {
    this.count += 1;
    cx.notify();
}))

// method-reference form (preferred for action handlers)
.on_action(cx.listener(Self::confirm))
// where: fn confirm(&mut self, _: &Confirm, &mut Window, &mut Context<Self>)
```
`cx.listener` captures a `WeakEntity<T>`, so the handler is safe even if the view is dropped (it no-ops). **This is the standard way every interaction updates state.**

## Events — `EventEmitter` + `cx.subscribe`

A view emits typed events; other views subscribe.

```rust
#[derive(Clone, Debug)]
enum CounterEvent { Incremented(usize) }

impl EventEmitter<CounterEvent> for Counter {}

// In the parent's new():
let _subscriptions: Vec<Subscription> = vec![
    cx.subscribe(&counter, |this, counter, event: &CounterEvent, cx| match event {
        CounterEvent::Incremented(n) => { /* react */ cx.notify(); }
    })
];
```
Store subscriptions in a `_subscriptions: Vec<Subscription>` field — **when a `Subscription` is dropped, the callback is deregistered**, so they must live as long as the view. The `_` prefix tells clippy you intend to keep them via the field, not read them.

## `Focusable`

```rust
impl Focusable for MyView {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        // either your own handle, or delegate to a child (e.g. an editor)
        self.focus_handle.clone()
    }
}
```
Create one with `let focus_handle = cx.focus_handle();` in `new()`. To track focus visually on an element: `.track_focus(&self.focus_handle)`.

## Actions — keyboard shortcuts + dispatchable commands

### Define an action
```rust
actions!(my_crate, [Save, Cancel]);           // zero-data actions
// or with data: #[derive(Action, ...)] struct Open { path: PathBuf }
```
Doc comments on actions are shown to the user in the command palette.

### Register a handler (two ways)

**A. On an element in `render()`** — fires when the element (or a focused descendant) receives the action:
```rust
v_flex()
    .id("my-view")                      // stateful — required for on_action
    .key_context("MyView")              // names the keybinding context
    .on_action(cx.listener(Self::save))
    .on_action(cx.listener(Self::cancel))
```
Handler signature: `fn save(&mut self, _: &Save, &mut Window, &mut Context<Self>)`.

**B. Globally on a Workspace** (for app-wide actions) — via `workspace.register_action(|workspace, action, window, cx| { ... })` inside a `cx.observe_new::<Workspace>(...)`.

### Dispatch an action
```rust
window.dispatch_action(Box::new(Save), cx);
// or from a focus handle:
focus_handle.dispatch_action(&Save, window, cx);
```

## Concurrency

All UI rendering and entity updates happen on a **single foreground thread**.
- `cx.spawn(async move |cx| { ... })` — async closure on the foreground thread (`cx: &mut AsyncApp`). For `Context<T>`: `cx.spawn(async move |this, cx| { ... })` where `this: WeakEntity<T>`.
- `cx.background_spawn(async move { ... })` — run on another thread; await from a foreground task to feed results back.
- Both return `Task<R>`. A dropped task is cancelled. To keep one alive: `.detach()`, `.detach_and_log_err(cx)`, or store it in a field.

```rust
cx.spawn(async move |this, cx| {
    let data = cx.background_spawn(async move { fetch_data().await }).await;
    this.update(cx, |this, cx| { this.data = data; cx.notify(); }).ok();
}).detach();
```

## `cx.notify()` — when to call it

Call `cx.notify()` after any state change that should affect rendering. It schedules a rerender and fires `cx.observe` callbacks. Forgetting it is the #1 "my state changed but the UI didn't" bug. You do **not** need it for changes made during `render` itself (those are obviously already reflected).

## A complete view template

This is the canonical shape of a GPUI view in this codebase — state, subscriptions, focus, actions, listener-wired interactions, and a render tree. Adapt it freely.

```rust
use gpui::{FocusHandle, IntoElement, Render, Window, actions};
use ui::prelude::*;

actions!(my_view, [Confirm, Cancel]);

struct MyView {
    value: SharedString,
    focus_handle: FocusHandle,
    _subscriptions: Vec<gpui::Subscription>,
}

impl MyView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        // observe / subscribe here, store into _subscriptions
        Self { value: "".into(), focus_handle, _subscriptions: vec![] }
    }

    fn confirm(&mut self, _: &Confirm, _window: &mut Window, cx: &mut Context<Self>) {
        if self.value.is_empty() { return; }
        // ... do work ...
        cx.notify();
    }

    fn cancel(&mut self, _: &Cancel, _window: &mut Window, cx: &mut Context<Self>) {
        self.value = "".into();
        cx.notify();
    }
}

impl Focusable for MyView {
    fn focus_handle(&self, _cx: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MyView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("my-view")
            .track_focus(&self.focus_handle)
            .key_context("MyView")
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .size_full()
            .p_4()
            .gap_3()
            .child(Label::new("Enter a value:"))
            .child(
                Button::new("confirm", "Confirm")
                    .disabled(self.value.is_empty())
                    .on_click(cx.listener(|this, _, window, cx| this.confirm(&Confirm, window, cx)))
            )
    }
}
```

## `cx.new` vs `cx.new(|cx| ...)` — note the `window`

In `new(window, cx)` you often create child entities: `cx.new(|cx| ChildView::new(window, cx))`. The inner closure receives `Context<ChildView>`. Pass `window` down if the child needs it at construction (most do — focus handles, editor setup).
