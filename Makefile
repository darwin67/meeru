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

.PHONY: rs-test
rs-test:
	cargo test $(RUST_MANIFEST)

.PHONY: up
up:
	docker-compose up -d

.PHONY: down
down:
	docker-compose stop
