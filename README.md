# Boltz

This repository is a cleaned Boltz Rust desktop application base. The main binary is a small GPUI app in `crates/boltz` with:

- GPUI windowing and direct element styling
- reusable `ui` components, `icons`, assets, and the Boltz theme registry
- database primitives through `db`, `sqlez`, and related support crates
- a single centered `hello world` startup view

Run the app on this macOS checkout with:

```sh
make dev
```

Use `runtime_shaders` when the machine does not have the full Xcode Metal toolchain available. For development checks:

```sh
make check
make fmt-check
```

The upstream product entrypoint, cloud/collab services, agent UI, release tooling, extensions, and deployment infrastructure have been removed. The retained workspace keeps the desktop base primitives needed for future app work.

Development notes live in `docs/desktop-base/`.
