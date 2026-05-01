# Boltz Base Guide

This base keeps the Boltz Rust desktop foundation without the upstream product surface.

## What Stays

- `crates/boltz`: the runnable desktop app.
- `crates/gpui*`: GPUI core, platform, and renderer runtime.
- `crates/ui`, `crates/component`, `crates/icons`: reusable GPUI UI components.
- `crates/assets`, `crates/theme`, `crates/syntax_theme`: fonts, icons, themes, and runtime theme support.
- `crates/db`, `crates/sqlez`, `crates/sqlez_macros`: local persistence primitives.
- `crates/boltz/src/main.rs`: the minimal window that renders centered `hello world` text through the shared UI layer.

## Run

```sh
make dev
```

The app should open a Boltz window with only `hello world` centered horizontally and vertically.

## Develop

Start feature work in `crates/boltz/src/main.rs` until a concept deserves its own crate. Keep new UI inside GPUI entities and prefer existing `ui` components before adding new primitives.

Use retained UI, theme, icon, asset, and database crates as the base toolbox for the new app.

## Editor Direction

The full old editor stack was product-coupled through workspace/project/LSP/git/debugger dependencies and has been removed from this base. Add a new editor surface only when the new product has stable editor requirements.
