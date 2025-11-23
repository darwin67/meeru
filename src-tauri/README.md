# Meeru Email Client - Rust Backend

This is the Tauri backend for the Meeru email client, implementing core email functionality.

## Development

### Building

```bash
# Check code compiles
cargo check

# Build for development
cargo build

# Build for production
cargo build --release
```

### Testing

**Important**: Integration tests require the `test-utils` feature flag to be enabled.

```bash
# Run all integration tests
cargo test --test integration_tests --features test-utils -- --test-threads=1

# Run specific test
cargo test --test integration_tests --features test-utils -- test_imap_connection --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test --test integration_tests --features test-utils -- --test-threads=1
```

**Note**:
- Tests require Docker (for testcontainers with Greenmail mail server)
- Must use `--test-threads=1` due to shared container state
- Some tests may fail without OS keychain access (see TESTING_STATUS.md)

### Why the `test-utils` feature?

The integration tests use simplified IMAP/SMTP clients (`imap_test`, `smtp_test`) that connect without TLS (required for Greenmail). These are gated behind the `test-utils` feature to:
1. Keep them out of production builds
2. Make it explicit when test-only code is compiled
3. Avoid accidentally using insecure connections in production

## Project Structure

```
src/
├── accounts/       # Account management + OS keychain
├── db/            # Database layer with SQLite
│   ├── migrations/ # SQL schema migrations
│   └── models.rs   # Database models
├── email/         # Email protocol implementations
│   ├── imap.rs        # Production IMAP client (TLS)
│   ├── smtp.rs        # Production SMTP client (TLS)
│   ├── sync.rs        # Email sync service
│   ├── imap_test.rs   # Test IMAP client (no TLS) [test-utils]
│   └── smtp_test.rs   # Test SMTP client (no TLS) [test-utils]
└── lib.rs         # Tauri app setup and commands

tests/
└── integration_tests.rs  # Integration tests with testcontainers
```

## Running the App

```bash
# Development mode (from project root)
make dev

# Or directly
pnpm tauri dev
```

## CI/CD Notes

When setting up CI, ensure:
1. Docker is available for integration tests
2. Use the test-utils feature: `cargo test --features test-utils`
3. Consider mocking keychain for sync tests (see TESTING_STATUS.md)

## Common Issues

### "could not find `imap_test` in `email`"
**Solution**: Add `--features test-utils` when running tests.

### "Failed to connect to Docker daemon"
**Solution**: Ensure Docker is running before running tests.

### "No matching entry found in secure storage"
**Solution**: This is expected in CI. See TESTING_STATUS.md for workarounds.

## Documentation

- `../PHASE1_IMPLEMENTATION.md` - Detailed Phase 1 implementation docs
- `../TESTING_STATUS.md` - Test results and known issues
- `../MVP.md` - Full MVP roadmap
