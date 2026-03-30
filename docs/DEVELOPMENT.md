# Development Setup Guide

This guide will help you set up your development environment for working on Meeru.

## Prerequisites

### All Platforms
- Rust 1.75 or higher
- Git
- SQLite 3

### Platform-Specific Requirements

#### Linux
```bash
# Ubuntu/Debian
sudo apt install libgtk-3-dev libssl-dev pkg-config

# Fedora
sudo dnf install gtk3-devel openssl-devel

# Arch
sudo pacman -S gtk3 openssl
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Windows
- Visual Studio 2019 or later with C++ tools
- Or: MSYS2 with mingw-w64 toolchain

## Setting Up the Development Environment

### 1. Clone the Repository
```bash
git clone https://github.com/darwin67/meeru.git
cd meeru
```

### 2. Install Rust (if not already installed)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Install Development Tools
```bash
# Install useful cargo extensions
cargo install cargo-watch    # Auto-rebuild on file changes
cargo install cargo-audit    # Security audit
cargo install cargo-outdated # Check for outdated deps
cargo install cargo-tarpaulin # Code coverage

# Or use the Makefile target
make install-dev-tools
```

### 4. Using Nix (Optional)
If you have Nix installed, you can use the provided flake for a reproducible dev environment:

```bash
# Enter the development shell
nix develop

# Or use direnv
direnv allow
```

## Building and Running

### Desktop Application
```bash
# Debug build
make dev-ui

# Release build
cargo build --release -p meeru-ui
```

### CLI Application
```bash
# Debug build
make dev-cli

# Run specific command
cargo run -p meeru-cli -- accounts list
```

### API Server
```bash
# Start the API server
make dev-api

# Or with custom port
cargo run -p meeru-cli -- serve --port 3000
```

## Testing

### Run All Tests
```bash
make test
```

### Run Tests with Coverage
```bash
make test-coverage
# Open target/tarpaulin-report.html in your browser
```

### Run Specific Test
```bash
cargo test -p meeru-core test_name
```

### Watch Mode (auto-run tests on file change)
```bash
cargo watch -x test
```

## Code Quality

### Formatting
```bash
# Check formatting
make fmt

# Auto-fix formatting
make fmt-fix
```

### Linting
```bash
# Run clippy
make lint

# Run with pedantic lints
cargo clippy -- -W clippy::pedantic
```

### Pre-commit Checks
Before committing, run:
```bash
make pre-commit
```

This will:
1. Format your code
2. Run lints
3. Run all tests

## Debugging

### VS Code
Add this to your `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Meeru UI",
            "cargo": {
                "args": [
                    "build",
                    "--package=meeru-ui",
                    "--bin=meeru"
                ],
                "filter": {
                    "name": "meeru",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Meeru CLI",
            "cargo": {
                "args": [
                    "build",
                    "--package=meeru-cli",
                    "--bin=meeru-cli"
                ],
                "filter": {
                    "name": "meeru-cli",
                    "kind": "bin"
                }
            },
            "args": ["accounts", "list"],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

### Logging
Set the `RUST_LOG` environment variable to control log levels:

```bash
# Show all logs
RUST_LOG=debug cargo run -p meeru-ui

# Show only meeru logs
RUST_LOG=meeru=debug cargo run -p meeru-ui

# Show specific module logs
RUST_LOG=meeru_core::sync=debug cargo run -p meeru-ui
```

## Architecture

See [docs/plans/technical-architecture.org](../plans/technical-architecture.org) for detailed architecture documentation.

## Database

### Migrations
Database migrations are handled automatically on startup. To create a new migration:

1. Add your migration to `meeru-storage/migrations/`
2. Update the migration runner in `meeru-storage/src/migrations.rs`

### Inspecting the Database
```bash
# Open SQLite CLI
sqlite3 ~/.local/share/meeru/meeru.db

# Show tables
.tables

# Show schema
.schema accounts
```

## Common Issues

### Linux: GTK errors
If you see GTK-related errors, ensure you have the development libraries installed (see Prerequisites).

### Windows: Link errors
Install Visual Studio with C++ tools or use MSYS2.

### macOS: SSL errors
Ensure you have OpenSSL installed via Homebrew:
```bash
brew install openssl
```

## Getting Help

- Check the [roadmap](../plans/roadmap.org) for current development status
- Look at existing issues on GitHub
- Ask in discussions or create an issue

## Tips for Contributors

1. **Small, focused commits** - Each commit should do one thing
2. **Write tests** - Add tests for new functionality
3. **Update documentation** - Keep docs in sync with code changes
4. **Follow conventions** - Use existing code style and patterns
5. **Run pre-commit checks** - Always run `make pre-commit` before pushing