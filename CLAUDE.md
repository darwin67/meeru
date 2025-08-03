# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

This is a Flutter project with the following development commands:

- `flutter run` - Run the app in development mode (also available via `make dev`)
- `flutter test` - Run all tests
- `flutter analyze` - Run static analysis and linting
- `flutter build apk` - Build Android APK
- `flutter build ios` - Build for iOS
- `flutter build web` - Build for web
- `flutter pub get` - Install dependencies
- `flutter pub upgrade` - Upgrade dependencies

## Project Architecture

This is a Flutter application with the following structure:

### Core Framework
- Uses **Flutter SDK 3.5.4+** as the primary framework
- Implements **ShadCN UI** (`shadcn_ui: ^0.28.3`) as the design system instead of Material Design
- The main app uses `ShadApp` rather than `MaterialApp`

### UI Architecture
- **ShadCN UI Integration**: The app uses ShadCN UI components with dark theme support
- **Theme Configuration**: Dark theme is configured with `ShadSlateColorScheme.dark()`
- **Component Structure**: Standard Flutter widget hierarchy with ShadCN components

### Project Structure
- `lib/main.dart` - Main application entry point with ShadCN UI setup
- `test/` - Widget and unit tests
- `android/`, `ios/`, `linux/`, `macos/`, `web/`, `windows/` - Platform-specific build configurations
- Standard Flutter multi-platform project structure

### Dependencies
- **UI Framework**: ShadCN UI for consistent design system
- **Icons**: Cupertino Icons for iOS-style icons
- **Linting**: Flutter Lints for code quality

### Development Environment
- Uses Nix flake for reproducible development environment (`flake.nix`, `flake.lock`)
- Standard Flutter analysis options configured in `analysis_options.yaml`
- No custom cursor rules or additional development configurations found

## Key Considerations

When working with this codebase:
- Always use ShadCN UI components instead of Material or Cupertino widgets where available
- The project supports all Flutter platforms (Android, iOS, Web, Linux, macOS, Windows)
- Follow the existing dark theme implementation patterns when adding new UI elements
- Run `flutter analyze` before committing to ensure code quality