RUST_MANIFEST=--manifest-path src-tauri/Cargo.toml

.PHONY: dev
dev:
	pnpm tauri dev

.PHONY: lint
lint:
	pnpm check --color
	cargo clippy $(RUST_MANIFEST) -- -D warnings

.PHONY: fmt
fmt:
	cargo fmt $(RUST_MANIFEST)
