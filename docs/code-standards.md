# Code Standards & Codebase Structure

## Edition & Toolchain

- **Rust Edition**: 2024
- **Toolchain**: Stable (no nightly features)
- **MSRV**: Stable toolchain supports edition 2024
- **Publish**: Enabled — workspace `publish = true`; crates namespaced `boltz-*` for crates.io (see scripts/publish-crates.sh)

## Formatting & Linting

### Rustfmt Configuration

**File**: `/rustfmt.toml`

```toml
edition = "2024"
style_edition = "2024"
```

All code must pass `cargo fmt` (2024 style).

### Format Check

```sh
make fmt-check
```

Runs `cargo fmt --all -- --check` to validate formatting without modifications.

## Code Organization (Per Crate)

### Standard Crate Structure

```
crates/example_crate/
├── src/
│   ├── lib.rs (or main.rs for binary)
│   ├── module_a.rs
│   ├── module_b/
│   │   ├── mod.rs
│   │   ├── submodule.rs
│   ├── tests/ (integration tests, if any)
├── examples/ (if applicable)
├── Cargo.toml
└── README.md (optional, document role if needed)
```

### Naming Conventions

- **Crate names**: kebab-case (`gpui_platform`, `font_kit`)
- **Module names**: snake_case (`text_system`, `metal_renderer`)
- **Type names**: PascalCase (`HelloWorldApp`, `ThemeSettingsProvider`)
- **Function/method names**: snake_case (`render`, `platform_facade`)
- **Constants**: UPPER_SNAKE_CASE (`APP_ID`, `DEFAULT_FONT_SIZE`)
- **Published names**: prefixed with `boltz-` in `Cargo.toml` (e.g., `package = "boltz-gpui"`)

## Workspace Conventions

### Adding a New Crate

1. Create `crates/new_feature/src/lib.rs` (or `main.rs` for binary)
2. Add to `[workspace]` `members` in `/Cargo.toml`
3. Define `Cargo.toml` with:
   - `edition = "2024"`
   - `version = "0.1.0"`
   - `package = "boltz-new-feature"` (if publishable)
   - `publish = false` (default)
4. Document role in `Cargo.toml` or crate comment
5. Add to workspace dependencies if other crates will depend on it

### Dependency Management

- Use workspace dependencies: declare in `/Cargo.toml` `[workspace.dependencies]`, import in crate via `workspace = true`
- Avoid dev-only dependencies in production crates when possible
- All crates share same dependency versions (no conflicts)

### Package Metadata

Each published crate includes:
```toml
[package]
name = "example"
version = "0.1.0"
edition = "2024"
publish = false
# or publish = true if intended for crates.io
```

Published crates use `package = "boltz-<name>"` for namespacing.

## GPUI Component Patterns

### Basic Component Structure

```rust
use gpui::{Render, IntoElement, Element};
use ui::prelude::*;

struct MyComponent {
    // state fields
}

impl Render for MyComponent {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .child(/* children */)
    }
}
```

### UI Component Library

When adding to `crates/ui`:
1. Follow existing component module structure (`src/components/label.rs`, etc.)
2. Implement `Render` trait for stateful components
3. Provide builder-style methods for configuration (e.g., `Label::new(text)`)
4. Use GPUI primitive styles (`flex`, `size`, `bg`, `text_color`, etc.)
5. Document with example usage in rustdoc

### Styling Approach

- **Direct styling**: use `div().bg(...).text_color(...).size(...)` 
- **No CSS classes**: GPUI uses builder API, not CSS
- **Themes**: pull colors from `theme::SystemColors` or `theme::UserTheme`
- **Flexbox**: use GPUI's `taffy` integration: `flex()`, `flex_col()`, `flex_row()`, `items_center()`, `justify_center()`, etc.

## Testing & Validation

### Build & Check Commands

```sh
make dev            # Run the app
make check          # cargo check (single crate)
make check-all      # cargo check --workspace (all targets, tests, benches)
make fmt-check      # Verify formatting
```

### Test Patterns

- **Real rendering**: use `TestAppContext` or `VisualTestContext` from GPUI for real window/render context
- **No mocks**: tests should drive real UI logic, not mock internals
- **Platform tests**: use `#[cfg(target_os = "...")]` only in platform-specific crates; app code avoids it

### CI Setup

**File**: `.github/workflows/publish.yml`

Runs on push/PR:
- `cargo check` with features
- `cargo fmt --check`
- `cargo clippy` (if configured)
- Publishes built artifacts to crates.io (on release tags)

## Platform-Specific Code

### Architecture

Platform abstraction is **not transparent**:
- Platform code lives in `gpui_macos`, `gpui_linux`, `gpui_windows` crates
- App code uses `gpui_platform::application()` facade (no `#[cfg]` gates)
- Platform-specific behavior (fonts, rendering API, windowing) isolated to backend crates

### When to Use Platform Gates

Allowed only in:
- `crates/gpui_platform/` (route OS detection)
- `crates/gpui_macos/`, `crates/gpui_linux/`, `crates/gpui_windows/` (platform-specific impl)
- `crates/font_kit/` (font loader selection per OS)

Never use in:
- `crates/app/` — always use platform facade
- `crates/ui/`, `crates/gpui/` — platform-agnostic only

## Feature Gates

### Commonly Used

- `gpui_platform/runtime_shaders` — use CPU-compiled shaders on macOS (avoids full Metal toolchain)
  ```sh
  cargo run -p app --features gpui_platform/runtime_shaders
  ```

### Adding New Features

Define in crate's `Cargo.toml`:
```toml
[features]
default = []
my_feature = ["dep:some_optional_crate"]
```

Enable via:
```sh
cargo run -p app --features my_feature
```

## Code Review Criteria

### Standards to Uphold

- **Formatting**: must pass `make fmt-check`
- **Build**: must pass `make check` and `make check-all`
- **Naming**: follow conventions (kebab-case for crates, snake_case for modules, PascalCase for types)
- **Scope**: features start in `crates/app`, split into new crate only when API is stable & reusable
- **Platform isolation**: no `#[cfg(target_os)]` in app code
- **Dependencies**: justify new workspace dependencies; prefer existing ones
- **Documentation**: public APIs should have rustdoc comments; complex logic should have inline comments

### Common Issues to Catch

- ❌ Unused dependencies (check with `cargo tree`)
- ❌ Cyclic crate dependencies (use `cargo tree` to detect)
- ❌ Platform gates in app-layer code
- ❌ Mocking in tests instead of using GPUI test context
- ❌ Breaking GPUI API changes without deprecation or docs

## Documentation Standards

### Crate Documentation

Each public crate should have:
1. **Module rustdoc**: `//!` comment at top of `lib.rs` explaining crate purpose
2. **Key type documentation**: public types/traits have `///` doc comments
3. **Example usage**: complex APIs include `# Example` sections in rustdoc

### Inline Comments

- Use `//` for explaining *why* (not *what* — code shows that)
- Use `// SAFETY:` before unsafe blocks explaining invariants
- Use `// TODO:` for known limitations (link to issue if applicable)

### README Files

Optional per-crate README for:
- Complex architecture (e.g., `crates/gpui_platform/README.md`)
- Integration instructions
- Platform-specific notes

## File Size Guidelines

- **Keep source files < 200 lines** where possible (aids readability, forces modularization)
- **Large modules**: split into submodules (e.g., `renderer.rs` → `renderer/mod.rs`, `renderer/atlas.rs`, `renderer/context.rs`)
- **Exception**: auto-generated code (e.g., build scripts) may exceed

## Cargo.toml Organization

Standard order:
```toml
[package]
name = "..."
version = "..."
edition = "2024"
publish = false

[dependencies]
# workspace deps first, then external
my_dep = { workspace = true }
external_dep = "1.0"

[dev-dependencies]
# test-only deps

[features]
# optional feature gates
```

## Makefile Targets

**File**: `/Makefile`

Current targets:
- `make dev` — run the app with `gpui_platform/runtime_shaders`
- `make check` — `cargo check` for main package
- `make check-all` — `cargo check --workspace --all-targets`
- `make fmt-check` — verify formatting

**To add a target**:
```makefile
new-target:
	@echo "Doing something"
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo <command>
```

## Version & Publishing

### Versioning

- **Workspace version**: "0.1.0" (initial template state; bump on breaking changes)
- **Per-crate version**: set per-crate if published separately
- **Semver**: follow semantic versioning (breaking changes = minor bump while <1.0.0)

### Publishing

- `publish = false` by default (template, not a library)
- To publish crate to crates.io:
  1. Set `publish = true` in `Cargo.toml`
  2. Ensure unique name (namespace with `boltz-` prefix)
  3. Update version
  4. Add to GitHub Actions publish workflow
  5. Tag release (e.g., `v0.1.0`)

## Build Configuration

**File**: `.cargo/config.toml`

Defines any custom build settings (e.g., linker, rustflags). Check file for project-specific overrides.

## Deprecated / Legacy Patterns

Avoid:
- Raw pointer arithmetic (use smart pointers, `Rc`, `Arc`)
- Unbounded generics (specify trait bounds)
- Partial moves in closures (use `move` or clone as needed)
- Implicit panics (use `Result` or `Option` for fallibility)
