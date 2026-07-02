# GPUI Codebase Research: Styling, Theme, Components, & Screenshot Capability

## 1. GPUI Styling Primitives

**File:** `crates/gpui/src/styled.rs` (via `Styled` trait) + `crates/ui/src/styles/*`

**Available methods on element** (builder pattern, all return `Self`):
- **Spacing:** `p_*`/`px_*/py_*`/`m_*`/`mx_*/my_*`/`gap_*`/`gap_x_*/gap_y_*` (rem ramp: 0, 0p5, 1–12, 16–96, `px`, `full`)
- **Size:** `w_*`/`h_*`/`size_*`/`min_w`/`max_w` (rem ramp + `full`/`auto`/fractional `1_2`, `1_3`, etc.)
- **Flexbox:** `flex()`/`flex_row()`/`flex_col()`/`flex_wrap()`, `items_start/center/end`, `justify_start/center/between`, `flex_grow_1()`/`flex_shrink_1()`
- **Color/Fill:** `bg(impl Into<Fill>)` (accepts `Hsla` or gradient), `text_color(Hsla)`, `border_color(Hsla)`
- **Border:** `border_0/1/2/4/8()`, `border_t/b/l/r/x/y_*()`, `border_dashed()`/`border_solid()`
- **Rounded corners:** `rounded_none/sm/md/lg/xl/2xl/3xl/full()`, per-corner variants `rounded_tl/tr/bl/br_*()`
- **Overflow:** `overflow_hidden()`, `overflow_x_hidden()`/`overflow_y_hidden()`, `overflow_scroll()`, `overflow_x_scroll()`/`overflow_y_scroll()`
- **Position:** `relative()`/`absolute()`/`fixed()`, `top/bottom/left/right_*()`, `inset_*()`
- **Text:** `text_color()`, `text_size()`, `text_xs/sm/base/lg/xl/2xl/3xl()`, `font_weight(FontWeight)`, `truncate()`, `text_ellipsis()`, `line_clamp(n)`
- **Visibility:** `visible()`/`invisible()`/`hidden()` (display:none), `cursor_pointer()`

**Limitations (missing Tailwind features):**
- ❌ `box-shadow` — NOT directly available (no `.shadow_*()` method). GPUI uses native platform shadows via `Elevation` + `elevation_index` separate attribute on `IntoElement`.
- ❌ CSS transitions/animations — NO animation property on elements; animations via `Animated` trait + property subscriptions (see `crates/ui/src/styles/animation.rs`).
- ❌ Focus ring — manual via border (no built-in `:focus-ring`); components handle via `.track_focus()` + `FocusHandle`.
- ❌ Gradients — `bg(Fill::Gradient(...))` exists but requires manual `Gradient` struct; not Tailwind-style `from_*/via_*/to_*` shortcuts.

**Source:** [file:crates/gpui/src/styled.rs](file:crates/gpui/src/styled.rs), [file:.claude/skills/gpui-ui-design/references/layout-styling.md](file:.claude/skills/gpui-ui-design/references/layout-styling.md)

---

## 2. Theme System & Color Tokens

**Files:** `crates/theme/src/theme.rs`, `crates/theme/src/schema.rs`, `crates/theme/src/default_colors.rs`, `crates/theme/src/fallback_themes.rs`

**Architecture:**
- `GlobalTheme` — holds active `Theme` via `cx.set_global()`. Access via `cx.theme()` (implements `ActiveTheme` trait on `App`).
- `Theme` struct — contains `colors: ThemeColors`, `styles`, `scale`, `icon_theme`.
- `ThemeColors` — semantic color struct with fields: `text`, `text_muted`, `element_background`, `border`, `border_variant`, `status` (success/warning/error), accent, UI colors.
- **Tailwind tokens insertion point:** `ThemeColors` can be extended with new fields (e.g., `palette: TailwindPalette`); each field maps to a named color. Theme JSON files load via `ThemeRegistry` (registry.rs).

**Current theming:**
- Default theme: `"One Dark"` (dark mode). Light mode available via `Appearance` enum.
- `LoadThemes::All(box_dyn_asset_source)` loads JSON theme files from bundled assets.
- `ThemeSettingsProvider` trait — app provides UI font, buffer font, font size, UI density (no direct color override yet, but design allows it).

**Wiring example (from main.rs):**
```rust
theme::init(LoadThemes::JustBase, cx);  // Initialize GlobalTheme
theme::set_theme_settings_provider(Box::new(provider), cx);  // Set font/density
cx.theme().colors().text  // Access color token
```

**Tokens insertion:** Add new color struct `struct TailwindPalette { slate_50, slate_100, ... }` → embed in `ThemeColors` → load from JSON or hardcode. No Tailwind CSS-style variable system exists; use Rust struct fields + pattern matching.

**Source:** [file:crates/theme/src/theme.rs](file:crates/theme/src/theme.rs), [file:crates/theme/src/schema.rs](file:crates/theme/src/schema.rs)

---

## 3. Component Pattern & Registry

**Component example:** [file:crates/ui/src/components/button/button.rs](file:crates/ui/src/components/button/button.rs)

**Pattern (builder + derive):**
```rust
#[derive(IntoElement, Documented, RegisterComponent)]
pub struct Button {
    base: ButtonLike,
    label: SharedString,
    // ... fields
}

impl Button {
    pub fn new(id: &str, label: impl Into<SharedString>) -> Self { ... }
    pub fn on_click(mut self, handler: impl Fn(...) + 'static) -> Self { ... }
    // ... builder methods
}
```

- `#[derive(IntoElement)]` — implements `IntoElement` trait (renders as GPUI element).
- `#[derive(RegisterComponent)]` — via `ui_macros::RegisterComponent` macro; **enables component registration for gallery preview**.
- Builder pattern: constructor returns `Self`, methods take `mut self`, return `Self`.
- Each component re-exports via `crates/ui/src/prelude.rs` → `use ui::prelude::*` brings all in.

**Component registry (`RegisterComponent` macro):**
- Macro defines `pub fn preview() -> impl IntoElement` on each component.
- Registry allows discovering all components + their preview functions (used for gallery/storybook).
- Source: `crates/ui_macros/src/lib.rs` (check grep output shows usage in button.rs, label.rs, count_badge.rs, etc.).

**Restyle workflow:** Follow this pattern—inherit from existing component, mutate builder methods. No CSS-in-JS; all styling via method calls in `render()` fn.

**Source:** [file:crates/ui/src/components/button/button.rs](file:crates/ui/src/components/button/button.rs), grep `RegisterComponent` in codebase

---

## 4. Screenshot Native: GPUI VisualTestContext

**File:** [file:crates/gpui/src/app/visual_test_context.rs](file:crates/gpui/src/app/visual_test_context.rs)

**Capability:**
- `VisualTestAppContext` — wraps real macOS Metal rendering (not mocked) + deterministic task scheduling.
- `open_offscreen_window()` — creates window at off-screen coords (-10000, -10000) for invisible rendering.
- Returns `WindowHandle<V>` → window is fully rendered by compositor, capturable via **ScreenCaptureKit** (macOS native).
- Output type: `image::RgbaImage` (see line 9: `use image::RgbaImage`).

**Mechanism:**
- Uses `VisualTestPlatform` wrapping real `MacPlatform`.
- `GpuiMode::test()` — special test mode enabling visual capture.
- No headless renderer abstraction (Metal only, platform-specific).

**Use case:** Loop screenshot component previews → validate rendered output against baseline images. Feasible but macOS-only (Windows/Linux need separate headless renderers).

**Source:** [file:crates/gpui/src/app/visual_test_context.rs](file:crates/gpui/src/app/visual_test_context.rs)

---

## 5. Workspace Wiring & Gallery Crate Integration

**File:** [file:Cargo.toml](file:Cargo.toml) (lines 1–34)

**Structure:**
```toml
[workspace]
members = [
    "crates/app",           # default-members
    "crates/gpui",
    "crates/ui",            # component crate
    "crates/theme",
    ...
]
default-members = ["crates/app"]
```

**Adding `examples/ui_gallery` crate:**
1. Create `examples/ui_gallery/Cargo.toml` with `[package] name = "ui_gallery"`.
2. Add to `Cargo.toml` members: `"examples/ui_gallery"`.
3. Keep `default-members = ["crates/app"]` → gallery won't build by default, only with explicit `cargo build -p ui_gallery`.
4. Dependency in gallery: `ui = { path = "../../crates/ui" }`, `theme = { path = "../../crates/theme" }`, `gpui = { path = "../../crates/gpui" }`.

**Bootstrap pattern (from main.rs lines 67–82):**
```rust
fn run_app() {
    application().run(|cx: &mut App| {
        theme::init(LoadThemes::JustBase, cx);
        theme::set_theme_settings_provider(Box::new(BaseThemeSettingsProvider::default()), cx);
        let bounds = Bounds::centered(None, size(px(640.0), px(420.0)), cx);
        let window = cx.open_window(WindowOptions { ... }, |_, cx| cx.new(|_| GalleryApp));
    });
}
```
Gallery reuses exact pattern: init theme → set provider → open window with gallery view.

**Component export:** All components already exported via `ui::prelude::*` (line 10 of button.rs imports `prelude`). Gallery imports once, reuses.

**Source:** [file:Cargo.toml](file:Cargo.toml), [file:crates/app/src/main.rs](file:crates/app/src/main.rs)

---

## Open Questions

1. **Tailwind palette generation:** Should UI kit define palette as Rust struct (more type-safe) or as theme JSON (more designer-friendly)? Codegen pipeline exists?
2. **Animation support:** GPUI's `Animated` trait + animation.rs — what's the scope? Can we do Tailwind-like `transition-all duration-200`?
3. **Focus ring standardization:** Every component needs custom focus ring via `track_focus()` + border. Can `RegisterComponent` macro codegen a standard focus ring?
4. **Screenshot loop performance:** VisualTestContext uses real Metal rendering — will looping 100+ component previews be fast enough, or need headless GPU rendering?
5. **Gradient syntax:** Tailwind `from-blue-500 to-purple-500` vs GPUI `bg(Fill::Gradient(...))`. Can macro generate gradient convenience methods?

---

## Trade-offs

| **Aspect** | **Option A** | **Option B** | **Note** |
|-----------|------------|------------|---------|
| **Color tokens** | Rust struct fields (`TailwindPalette`) | JSON theme files | Struct = type-safe, JSON = hot-reload-friendly |
| **Shadow/elevation** | Use `elevation_index` (native) | Fake shadows via `border` + `rounded` | Native wins on mobile, border is hackable |
| **Animation** | Lean on Animated trait | Pre-build static transitions CSS | Trait = powerful, CSS = familiar |
| **Gallery tech** | VisualTestContext loop | Custom web-based storybook | GPUI loop = native preview, web = designer-friendly |
| **Component discovery** | RegisterComponent macro | Manual registry file | Macro = zero-maintenance, registry = explicit |

---

## Technical Risks

1. **Missing shadow API:** Box-shadow not exposed as `.shadow_*()` — gallery must document elevation index or custom workaround.
2. **macOS-only headless rendering:** VisualTestContext Metal only; Windows/Linux need WGPU headless (not yet exposed cleanly).
3. **Tailwind familiarity gap:** Rust builder pattern != CSS class names; docs must teach translation.
4. **Theme JSON coupling:** Extending `ThemeColors` requires both Rust struct AND JSON updates; easy to diverge.
