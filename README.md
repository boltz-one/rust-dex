# Desktop App Template

A minimal, cross-platform Rust desktop application template built on [GPUI](https://gpui.rs). The main binary is a small GPUI app in `crates/app` that opens a single centered `hello world` window.

Highlights:

- GPUI windowing with direct element styling and flexbox layout
- Reusable `ui` components, `icons`, and a theme system with built-in fallback themes
- Cross-platform native backends: macOS (Metal), Linux (wgpu/Wayland/X11), Windows (DirectX)
- No bundled assets — system fonts and built-in themes keep the template lean
- No persistence layer — re-add when the app needs state

Run the app on this macOS checkout with:

```sh
make dev
```

`make dev` uses the `gpui_platform/runtime_shaders` feature, which avoids requiring the full Xcode Metal toolchain. For development checks:

```sh
make check
make fmt-check
```

Development notes live in `docs/`.

## Reference

The `app/` directory contains the upstream App editor source kept only as a reference; it is gitignored and not part of the workspace.
