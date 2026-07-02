# font_kit (vendored)

A cross-platform font loading, matching, and rasterization library. This is a
**vendored copy** of Zed's fork of [servo/font-kit], kept in-tree so the
workspace has no network/git dependency for font support.

## Provenance

- Upstream: <https://github.com/zed-industries/font-kit> (package `zed-font-kit`)
- Pinned revision: `94b0f28166665e8fd2f53ff6d268a14955c82269` (`0.14.1-zed`)
- License: MIT OR Apache-2.0 (see `LICENSE-MIT` / `LICENSE-APACHE`)

## Consumer

Used only by `gpui_macos` (Core Text backend) for glyph rasterization. The crate
remains cross-platform — the Core Text, DirectWrite, and FreeType/Fontconfig
backends are all present and selected by `cfg` per target, exactly as upstream.

## What changed when vendoring

The Rust sources are **behaviorally unchanged** from the pinned revision. The
only source edits are whitespace / import-ordering applied by the workspace's
`rustfmt` (2024 edition) so `cargo fmt --all -- --check` passes — there are no
logic changes. Non-source artifacts were dropped:

- `resources/`, `c/`, `examples/`, `tests/` — test fixtures, C bindings, demos
- `.git/`, `.github/`, `.gitignore` — VCS/CI metadata
- `[dev-dependencies]` (clap, colored, pbr, prettytable-rs) — only used by the
  removed examples/tests

The package was renamed `zed-font-kit` → `font_kit` and marked `publish = false`.

## Updating

To re-sync with a newer upstream revision:

1. Re-clone at the new rev and copy `src/` + `build.rs` over this copy.
2. Re-apply the `Cargo.toml` trim above (package rename, drop dev-deps).
3. Run `cargo fmt --all` so the vendored sources match the workspace style.

[servo/font-kit]: https://github.com/servo/font-kit
