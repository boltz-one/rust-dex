# Layout & Styling — `div()`, flex, and the Tailwind-ish method families

Source of truth: `crates/gpui/src/elements/div.rs`, `crates/gpui/src/styled.rs`, `crates/gpui_macros/src/styles.rs`, `crates/ui/src/components/stack.rs`, `crates/ui/src/components/group.rs`.

## The element: `div()` and `Div`

`div()` returns a `Div` — the universal layout container. `Div` implements:
- `Styled` — all the Tailwind-ish style methods (padding, gap, flex, color, …).
- `ParentElement` — `.child(impl IntoElement)`, `.children(impl IntoIterator<Item = impl IntoElement>)`.
- `InteractiveElement` — but `on_click`/`on_hover`/`on_action`/`track_focus` require the element to be **stateful**.
- `IntoElement` — a `Div` is already an element (`.into_element()` is a no-op-ish identity).

**Stateful vs stateless:** a plain `div()` can take `.child(...)`, style methods, and `.when(...)`, but NOT `on_click`/`on_hover`/`on_action`. To get those, call `.id("some-id")` — it returns a `Stateful<Div>`. This is the single most common compile error: "no method `on_click` on `Div`". Fix: add `.id("...")`.

```rust
// Stateless — fine for pure layout:
div().flex().gap_2().p_4().child(Label::new("hi"))

// Stateful — needed for interaction:
div().id("clickable").on_click(|_, window, cx| { /* ... */ }).child(Label::new("click me"))
```

Note: components like `Button`/`IconButton` are already interactive (they wrap a `ButtonLike` with an id), so you don't add `.id()` to them — just call `.on_click(...)` directly.

## Layout primitives (use these first)

### `h_flex()` / `v_flex()` — the bread and butter
```rust
// crates/ui/src/components/stack.rs (via traits/styled_ext.rs)
pub fn h_flex() -> Div   // = div().flex().flex_row().items_center()
pub fn v_flex() -> Div   // = div().flex().flex_col()
```
- `h_flex` = horizontal row, cross-axis **centered**.
- `v_flex` = vertical column, no cross-axis alignment set.
- **Neither sets a gap.** You must add `.gap_N()` yourself. Forgetting the gap is the most common "why is everything touching" bug.

### `h_group` / `v_group` (+ `_sm` / `_lg` / `_xl`) — flex + a fixed gap
```rust
// crates/ui/src/components/group.rs
h_group_sm()   // gap_0p5  (~2px @16rem)
h_group()      // gap_1    (~4px)
h_group_lg()   // gap_1p5  (~6px)
h_group_xl()   // gap_2    (~8px)
v_group_sm() / v_group() / v_group_lg() / v_group_xl()   // same gaps, flex_col
```
Difference vs `h_flex`/`v_flex`: groups bake in a sized gap; `h_group` does **not** set `items_center` (flex default is row). Use groups for rows/columns of controls with consistent spacing; use `h_flex`/`v_flex` when you want centered alignment or a custom gap.

## The rem ramp (what `p_2`, `gap_4`, `w_96` mean)

Numeric suffixes are a rem scale where the rem size comes from the theme's UI font size (default 16px → 1rem = 16px; at 14px UI font, 1rem = 14px). The scale (suffix → rems):
```
0 → 0     0p5 → 0.5    1 → 1   1p5 → 1.5   2 → 2   2p5 → 2.5
3 → 3     3p5 → 3.5    4 → 4   5 → 5       6 → 6   7 → 7
8 → 8     9 → 9        10 → 10 11 → 11     12 → 12 16 → 16
20 → 20   24 → 24      32 → 32 40 → 40     48 → 48 56/64/72/80/96
px → 1px   full → 100%   auto → auto
fractional: 1_2 → 50%  1_3/2_3  1_4/2_4/3_4  1_5
```
So `p_2` = 0.5rem padding (~8px), `gap_4` = 1rem (~16px), `w_96` = 24rem (~384px). `p_0p5` = 2px. Methods taking a literal value also exist: `p(DefiniteLength)`, `gap(len)`, `w(len)`.

## Method families (all return `Self` — builder pattern)

### Padding / margin
`p_*` (all), `px_*` (x), `py_*` (y), `pt/pb/pl/pr_*` (side). Margins mirror: `m_*`, `mx/my/mt/mb/ml/mr_*`, `m_auto()`. Both accept the full rem ramp + `px`/`full`/`auto`/fractional.

### Size / width / height
`w_*`, `h_*`, `size_*` (both), `min_w/min_h_*`, `max_w/max_h_*`, `min_size/max_size_*`. Plus `w_full()`/`h_full()`/`size_full()`, `w_auto()`, fractional (`w_1_2`). Custom: `w(impl Into<DefiniteLength>)`.

### Gap
`gap_*`, `gap_x_*`, `gap_y_*`. Custom: `gap(len)`.

### Flexbox
Direction: `flex()`, `flex_row()`, `flex_row_reverse()`, `flex_col()`, `flex_col_reverse()`.
Grow/shrink/basis: `flex_1()` (grow+shrink+basis 0), `flex_auto()`, `flex_none()`, `flex_initial()`, `flex_grow(f32)` / `flex_grow_0()` / `flex_grow_1()`, `flex_shrink(f32)` / `flex_shrink_0()` / `flex_shrink_1()`, `flex_basis(len)`, `flex_wrap()` / `flex_wrap_reverse()` / `flex_nowrap()`.
Align items: `items_start/end/center/baseline/stretch`.
Align self: `self_start/end/flex_start/flex_end/center/baseline/stretch`.
Align content: `content_normal/center/start/end/between/around/evenly/stretch`.
Justify: `justify_start/end/center/between/around/evenly()`.

### Background / fill
`bg(impl Into<Fill>)` — accepts `Hsla` or a `Fill` (gradient). Feed it a token, not a raw theme
call or literal: `bg(semantic::surface(cx))` (neutral) or `bg(palette::primary(600))` (accent).
`bg_transparent()`.

### Border
Width: `border_0/1/2/4/8()`, per-side `border_t/b/l/r/x/y_N()`. Color: `border_color(Hsla)` —
use `semantic::border(cx)` / `semantic::border_muted(cx)` / `semantic::border_focused(cx)`, not a
hardcoded `Hsla`. Style: `border_dashed()`, `border_solid()`.

### Rounded corners
`rounded_none/sm/md/lg/xl/2xl/3xl/full()`, per-corner `rounded_t/b/l/r/tr/tl/br/bl_*()`. `rounded_full()` = pill.

### Overflow
`overflow_hidden()`, `overflow_x_hidden()` / `overflow_y_hidden()`, `overflow_scroll()`, `overflow_x_scroll()` / `overflow_y_scroll()`. Use `overflow_hidden()` + `min_w_0()` together to let a flex child truncate text (see `Button::truncate`).

### Position
`relative()`, `absolute()`, `fixed()`. Insets: `top/bottom/left/right_*()`, `inset_*()`. For overlays anchored inside a `relative()` parent, use `absolute().top_0().right_0()`.

### Visibility / cursor / display
`visible()`, `invisible()`, `hidden()` (display none), `block()`, `flex()`, `grid()`. `cursor_pointer()`, `cursor_default()`, etc.

### Text (on `div`/`Styled`; prefer `Label` for text content)
`text_left/center/right()`, `truncate()`, `text_ellipsis()`, `line_clamp(n)`, `text_color(Hsla)`, `text_size(len)`, `text_xs/sm/base/lg/xl/2xl/3xl()`, `font_weight(FontWeight)`, `font_family(name)`, `font(Font)`. **For actual text, use `Label`** — it handles font, color, truncation, and code spans properly.

## Conditional builders (essential GPUI idiom)

`.when`/`.when_some`/`.when_else` let you keep the builder chain while branching — far cleaner than `if`/`else` building two separate trees.

```rust
div()
    .when(self.has_error, |this| this.border_color(palette::danger(500)))
    .when_some(self.icon, |this, icon| this.child(icon))   // runs only if Some
    .when_some(self.count, |this, n| this.child(Label::new(format!("{n}"))))
    .when_else(is_loading,
        |this| this.child(Spinner),
        |this| this.child(Button::new("go", "Go")),
    )
```
- `.when(cond: bool, |this| this)` — runs closure only when `cond`.
- `.when_some(opt: Option<T>, |this, value| this)` — runs only when `Some(value)`.
- `.when_else(cond, then_fn, else_fn)` — both branches must return the same builder type.

These come from the `FluentBuilder` trait and work on `Div`, `Stateful<Div>`, and most components.

## Children

- `.child(impl IntoElement)` — one child. `String`/`&str`/`SharedString`/`Label`/`Icon`/`Button`/`Div` all implement `IntoElement`.
- `.children(impl IntoIterator<Item = impl IntoElement>)` — many. Use with `.map(|x| ...)` to render a `Vec`.
- `.into_any_element()` / `.into_any()` — erase the concrete type so you can return different element types from branches (common when a view's `render()` early-returns `div().hidden().into_any_element()`).

## Putting it together — a panel skeleton

```rust
use ui::prelude::*;

v_flex()
    .size_full()
    .bg(semantic::surface(cx))
    .border_1()
    .border_color(semantic::border(cx))
    .rounded_md()
    .overflow_hidden()
    // Header row
    .child(
        h_flex().w_full().px_4().py_3().gap_2().border_b_1()
            .border_color(semantic::border_muted(cx))
            .child(Icon::new(IconName::Settings).size(IconSize::Small))
            .child(Label::new("Settings").weight(FontWeight::BOLD))
            .child(div().flex_1())  // spacer pushes the rest right
            .child(IconButton::new("close", IconName::Close).style(ButtonStyle::Subtle))
    )
    // Body
    .child(
        v_flex().p_4().gap_4()
            .child(List::new()
                .child(ListItem::new("a").child(Label::new("Option A")))
                .child(ListItem::new("b").child(Label::new("Option B"))))
    )
    .when(self.has_footer, |this| this.child(
        h_flex().p_3().border_t_1().border_color(semantic::border_muted(cx))
            .child(Button::new("save", "Save").full_width())
    ))
```

## Tips
- A `div().flex_1()` with no children is the idiomatic **spacer** in an `h_flex`/`v_flex` (pushes siblings apart with `justify_between`-like effect).
- For text that must truncate inside a flex row: parent `.overflow_hidden().min_w_0()` + `Label::new(...).truncate()`. The `min_w_0` lets the flex child shrink below its content width.
- `size_full()` = `w_full() + h_full()` — fills the parent. Pair with a `relative()` parent for overlays.
- Elevation: `.elevation_2(cx)` / `.elevation_3(cx)` apply elevated surface bg + shadow (modals use `elevation_3`).
