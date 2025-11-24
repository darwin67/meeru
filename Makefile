.PHONY: dev
dev:
	pnpm tauri dev


.PHONY: lint
lint:
	pnpm check --color
	cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
