RUSTUP_TOOLCHAIN ?= stable
RUN_FEATURES := gpui_platform/runtime_shaders
PACKAGE := app

.PHONY: dev check fmt-check

dev:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo run -p $(PACKAGE) --features $(RUN_FEATURES)

check:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo check -p $(PACKAGE) --features $(RUN_FEATURES)

fmt-check:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo fmt --all -- --check
