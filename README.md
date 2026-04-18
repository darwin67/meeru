# Meeru Email Client

A modern, unified desktop email client built with Rust that provides transparent multi-account management, AI-powered features, and a CLI interface for automation.

## Features

- 🔄 **Unified Multi-Account View** - Seamlessly manage multiple email accounts with unified folders, labels, and operations
- 🤖 **AI-Powered Features** - Local LLM support for email categorization, summarization, and smart compose
- 💻 **Cross-Platform** - Native desktop app for Windows, macOS, and Linux
- 🔧 **CLI & API** - Command-line interface and local API for automation and AI agents
- 🔐 **Privacy-First** - All AI processing can run locally, your data never leaves your device
- 🚀 **High Performance** - Built with Rust for speed and reliability

## Project Structure

This is a Rust workspace with the following crates:

- `meeru-core` - Core business logic and email operations
- `meeru-ui` - Desktop UI built with Iced
- `meeru-cli` - Command-line interface and API server
- `meeru-storage` - Data persistence layer (SQLite, file storage, search)
- `meeru-providers` - Email provider adapters (IMAP, SMTP, OAuth)
- `meeru-ai` - AI/ML features for email processing

## Getting Started

### Prerequisites

- Rust 1.75 or higher
- SQLite 3
- On Linux: GTK3 development libraries

### Building

```bash
# Clone the repository
git clone https://github.com/darwin67/meeru.git
cd meeru

# Build all components
make build

# Run tests
make test

# Run the desktop app
make dev-ui

# Run the CLI
make dev-cli
```

### Desktop Installers

Build installer artifacts for the current host platform:

```bash
make package-desktop
```

Platform-specific targets:

```bash
make package-macos
make package-linux
make package-windows
```

Expected outputs:

- macOS: `.app` bundle and `.dmg` in `target/release/bundle/osx/`
- Linux: `.deb` and `.AppImage` in `target/release/bundle/`
- Windows: `.msi` in `target/release/bundle/msi/`

The tag-based GitHub release workflow also builds these desktop installers on native GitHub runners and uploads them to the GitHub release assets.

### Development

```bash
# Format code
make fmt-fix

# Run lints
make lint

# Run pre-commit checks
make pre-commit

# Generate documentation
make docs
```

## Roadmap

See the project overview in [docs/project-overview.org](docs/project-overview.org) and the numbered development plans under [docs/plans](docs/plans), starting with [docs/plans/001-project-setup-and-architecture.org](docs/plans/001-project-setup-and-architecture.org).

### Phase 1: Core Foundation (Current)
- [x] Project setup and architecture
- [ ] Basic storage layer
- [ ] IMAP/SMTP integration
- [ ] Basic UI implementation

### Phase 2: Unified Experience
- [ ] Multi-account management
- [ ] Unified folder system
- [ ] Cross-account search
- [ ] Contact unification

### Phase 3: Provider Integration
- [ ] Gmail integration
- [ ] Outlook support
- [ ] OAuth authentication

### Phase 4: AI Features
- [ ] Local LLM integration
- [ ] Email categorization
- [ ] Smart compose
- [ ] Priority detection

### Phase 5: CLI & Automation
- [ ] Complete CLI interface
- [ ] Local API server
- [ ] Agent integration

## Contributing

Contributions are welcome! Please read our contributing guidelines and code of conduct before submitting PRs.

## License

This project is dual-licensed under MIT and Apache-2.0 licenses.
