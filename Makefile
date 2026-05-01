RUSTUP_TOOLCHAIN ?= stable
RUN_FEATURES := gpui_platform/runtime_shaders
PACKAGE := boltz
WEB_TARGET := wasm32-unknown-unknown
WEB_DIR := crates/boltz
WEB_INDEX := index.html
TRUNK ?= trunk

.PHONY: dev check fmt-check web web-build web-check web-target

dev:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo run -p $(PACKAGE) --features $(RUN_FEATURES)

check:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo check -p $(PACKAGE) --features $(RUN_FEATURES)

fmt-check:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo fmt --all -- --check

web-target:
	rustup target add --toolchain $(RUSTUP_TOOLCHAIN) $(WEB_TARGET)

web: web-target
	command -v $(TRUNK) >/dev/null || { echo "trunk is required. Install with: cargo install trunk"; exit 1; }
	cd $(WEB_DIR) && RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) RUSTC_BOOTSTRAP=1 $(TRUNK) serve $(WEB_INDEX)

web-build: web-target
	command -v $(TRUNK) >/dev/null || { echo "trunk is required. Install with: cargo install trunk"; exit 1; }
	cd $(WEB_DIR) && RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) RUSTC_BOOTSTRAP=1 $(TRUNK) build $(WEB_INDEX)

web-check: web-target
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) RUSTC_BOOTSTRAP=1 cargo check --target $(WEB_TARGET) -p $(PACKAGE)
