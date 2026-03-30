.PHONY: dev dev-cli dev-ui lint fmt test clean build build-release check

## Run the desktop application in debug mode
dev: dev-ui

## Run the CLI in debug mode
dev-cli:
	cargo run -p meeru-cli -- --help

## Run the desktop UI in debug mode
dev-ui:
	cargo run -p meeru-ui

## Run the API server
dev-api:
	cargo run -p meeru-cli -- serve

## Run clippy lints on the workspace
lint:
	cargo clippy --workspace --all-targets -- -D warnings

## Format all Rust source files
fmt:
	cargo fmt --all -- --check

## Format and fix all Rust source files
fmt-fix:
	cargo fmt --all

## Run all tests
test:
	cargo test --workspace

## Run tests with coverage (requires cargo-tarpaulin)
test-coverage:
	cargo tarpaulin --workspace --out Html

## Clean build artifacts
clean:
	cargo clean

## Build all binaries in debug mode
build:
	cargo build --workspace

## Build all binaries in release mode
build-release:
	cargo build --workspace --release

## Check if project compiles
check:
	cargo check --workspace

## Run benchmarks (when available)
bench:
	cargo bench --workspace

## Generate documentation
docs:
	cargo doc --workspace --no-deps --open

## Install development tools
install-dev-tools:
	cargo install cargo-tarpaulin
	cargo install cargo-audit
	cargo install cargo-outdated

## Check for security vulnerabilities
audit:
	cargo audit

## Check for outdated dependencies
outdated:
	cargo outdated

## Update dependencies
update:
	cargo update

## Run pre-commit checks
pre-commit: fmt-fix lint test

## Print help
help:
	@echo "Available targets:"
	@grep -E '^##' Makefile | sed -E 's/^## /  /'