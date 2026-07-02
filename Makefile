RUSTUP_TOOLCHAIN ?= stable
RUN_FEATURES := gpui_platform/runtime_shaders
PACKAGE := app

.PHONY: dev check check-all fmt-check

dev:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo run -p $(PACKAGE) --features $(RUN_FEATURES)

check:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo check -p $(PACKAGE) --features $(RUN_FEATURES)

# Check the whole workspace including tests, examples, and benches.
# Catches regressions where the app still builds but the test suite is broken.
check-all:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo check --workspace --all-targets --features $(RUN_FEATURES)

fmt-check:
	RUSTUP_TOOLCHAIN=$(RUSTUP_TOOLCHAIN) cargo fmt --all -- --check
