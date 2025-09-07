# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

This is a Flutter project with the following development commands:

- `flutter run` - Run the app in development mode (also available via `make dev`)
- `flutter test` - Run all tests
- `flutter analyze` - Run static analysis and linting (also available via `make lint`)
- `dart format .` - Format code (also available via `make fmt`)
- `flutter clean` - Clean build artifacts (also available via `make clean`)
- `dart_depcheck` - Check for dependency issues (also available via `make depcheck`)
- `flutter build apk` - Build Android APK
- `flutter build ios` - Build for iOS
- `flutter build web` - Build for web
- `flutter pub get` - Install dependencies
- `flutter pub upgrade` - Upgrade dependencies

## Project Architecture

**Meeru** is a cross-platform Flutter email client application designed as a modern, secure email client supporting multiple email providers with OAuth and password authentication.

### Core Framework & Technology Stack
- **Flutter SDK**: 3.5.4+ as the primary framework
- **UI Framework**: **ShadCN UI** (`shadcn_ui: ^0.28.3`) instead of Material Design
  - Uses `ShadApp` rather than `MaterialApp` as the root widget
  - Dark theme support with `ShadSlateColorScheme.dark()`
  - Always use ShadCN UI components instead of Material or Cupertino widgets where available
- **State Management**: Provider pattern (`provider: ^6.1.2`) with `ChangeNotifier`
- **HTTP Client**: `dio: ^5.4.3` and `http: ^1.2.0` for networking and OAuth flows
- **Secure Storage**: `flutter_secure_storage: ^9.2.2` for credential management
- **Platform Support**: Full multi-platform (Android, iOS, Web, Linux, macOS, Windows)
- **Development Environment**: Nix flake for reproducible development environment

### Application Architecture

#### State Management Pattern
The app uses Provider pattern with a centralized `AuthProvider` that manages:
- Authentication state and loading states
- Account loading, switching, and removal
- Integration with storage services
- Token refresh automation
- Error handling across the app

#### Service Layer Architecture
- **EmailAuthService**: Authentication orchestration for both password and OAuth, server connection testing, account creation, token refresh handling
- **CredentialStorageService**: Secure credential persistence, account list management, cross-platform storage abstraction

#### Security Implementation
- **Secure Storage**: Platform-specific encrypted storage (Keychain on iOS/macOS, Encrypted SharedPreferences on Android, IndexedDB with encryption on Web)
- **OAuth 2.0 Support**: Full OAuth flow implementation for Gmail and Outlook with automatic token refresh
- **Credential Isolation**: Separate storage keys per account
- Never store passwords in plain text, all credentials are encrypted

### Data Models

#### EmailAccount Model
- Comprehensive account representation with IMAP/SMTP configurations
- Support for multiple providers (Gmail, Outlook, Yahoo, iCloud, Custom)  
- JSON serialization for persistent storage
- Immutable pattern with `copyWith` methods

#### EmailCredentials Model
- Secure credential storage for passwords and OAuth tokens
- Token expiration tracking and refresh logic
- Support for both password and OAuth authentication methods

#### ProviderConfig Model
- Predefined configurations for major email providers
- OAuth configuration with scopes and endpoints
- Server configurations (IMAP/SMTP settings)
- Extensible for custom providers

### UI/Navigation Patterns
- **Page-based Navigation**: Uses `PageRouteBuilder` with custom fade transitions (not `MaterialPageRoute`)
- **Form Validation**: Real-time validation with proper error handling
- **Responsive Design**: Supports all Flutter platforms with responsive layouts
- **Theme Support**: Consistent dark theme implementation patterns

### Current Development Status
The codebase is in **Phase 1** of the MVP (see docs/MVP.md) with the following implemented:
- ✅ Account setup and authentication flows
- ✅ Secure credential storage  
- ✅ Multi-provider support (Gmail, Outlook, Yahoo, iCloud, Custom)
- ✅ OAuth 2.0 integration framework
- ✅ Account management UI
- ✅ Provider-based state management
- ✅ Cross-platform secure storage

**Next Phase**: Email fetching, display, and IMAP/SMTP integration

### File Structure & Key Components
```
lib/
├── main.dart                    # ShadApp root with AuthProvider integration
├── models/                      # Data models with JSON serialization
│   ├── email_account.dart       # Account representation & IMAP/SMTP config
│   ├── email_credentials.dart   # Secure credential storage model
│   └── provider_config.dart     # Email provider configurations
├── providers/                   # State management
│   └── auth_provider.dart       # Central authentication state
├── screens/                     # UI screens using ShadCN UI
│   ├── welcome_screen.dart      # Onboarding with feature highlights
│   ├── account_list_screen.dart # Account management interface
│   └── account_setup_screen.dart # Account configuration wizard
└── services/                    # Business logic services
    ├── credential_storage_service.dart # Cross-platform secure storage
    └── email_auth_service.dart  # Authentication & server connection
```

## Key Development Guidelines

### Navigation
- Always use `PageRouteBuilder` with custom transitions, not `MaterialPageRoute`
- Use `Navigator.pushReplacement()` for authentication flows to prevent back navigation to auth screens

### Authentication Flow
- Account setup follows: Provider selection → Credential entry → Server testing → Storage
- OAuth flows use authorization code exchange with proper token storage
- Always test IMAP/SMTP connections before account creation
- Handle token refresh automatically in background

### Error Handling
- Use custom exceptions in services layer
- Provide user-friendly error messages in UI
- Handle network failures gracefully
- Log errors appropriately without exposing sensitive information

### Security Requirements
- All credentials stored via `flutter_secure_storage`
- OAuth tokens automatically refresh before expiration  
- Validate all user inputs before processing
- Use TLS/SSL for all server connections
- Never log or expose credentials in error messages

### Code Quality
- Run `flutter analyze` before committing to ensure code quality
- Follow existing dark theme implementation patterns
- Use Provider pattern for state management consistently
- Implement proper loading states and error handling in UI