## Run the desktop application in debug mode
.PHONY: dev
dev: dev-ui

## Run the CLI in debug mode
.PHONY: dev-cli
dev-cli:
	cargo run -p meeru-cli -- --help

## Run the desktop UI in debug mode
.PHONY: dev-ui
dev-ui:
	cargo run -p meeru-ui

## Run the API server
.PHONY: dev-api
dev-api:
	cargo run -p meeru-cli -- serve

## Run clippy lints on the workspace
.PHONY: lint
lint:
	cargo clippy --workspace --all-targets -- -D warnings

## Format all Rust source files
.PHONY: fmt
fmt:
	cargo fmt --all -- --check

## Format and fix all Rust source files
.PHONY: fmt-fix
fmt-fix:
	cargo fmt --all

## Run all tests
.PHONY: test
test:
	cargo test --workspace

## Run tests with coverage (requires cargo-tarpaulin)
.PHONY: test-coverage
test-coverage:
	cargo tarpaulin --workspace --out Html

## Clean build artifacts
.PHONY: clean
clean:
	cargo clean

## Build all binaries in debug mode
.PHONY: build
build:
	cargo build --workspace

## Build all binaries in release mode
.PHONY: build-release
build-release:
	cargo build --workspace --release

## Check if project compiles
.PHONY: check
check:
	cargo check --workspace

## Run benchmarks (when available)
.PHONY: bench
bench:
	cargo bench --workspace

## Generate documentation
.PHONY: docs
docs:
	cargo doc --workspace --no-deps --open

## Check plan metadata and checklist status
.PHONY: plan-status
plan-status:
	bash scripts/check-plan-status.sh

## Install development tools
.PHONY: install-dev-tools
install-dev-tools:
	cargo install cargo-tarpaulin
	cargo install cargo-audit
	cargo install cargo-outdated

## Check for security vulnerabilities
.PHONY: audit
audit:
	cargo audit

## Check for outdated dependencies
.PHONY: outdated
outdated:
	cargo outdated

## Update dependencies
.PHONY: update
update:
	cargo update

## Run pre-commit checks
.PHONY: pre-commit
pre-commit: fmt-fix lint test

## Print help
.PHONY: help
help:
	@echo "Available targets:"
	@grep -E '^##' Makefile | sed -E 's/^## /  /'
