RUSTUP_TOOLCHAIN ?= stable
RUN_FEATURES := gpui_platform/runtime_shaders
PACKAGE := ui_gallery

.PHONY: dev check test fmt-check

dev:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo run -p $(PACKAGE) --features $(RUN_FEATURES)

check:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo check -p $(PACKAGE) --all-targets --features $(RUN_FEATURES)

test:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo test -p $(PACKAGE) --features $(RUN_FEATURES)

fmt-check:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo fmt --all -- --check
