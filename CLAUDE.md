# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Meeru is a cross-platform email application built with Tauri v2, SvelteKit, and TypeScript. The architecture follows a hybrid desktop app pattern where:
- **Frontend**: SvelteKit using static adapter (SSG) with Svelte 5, TypeScript, and Tailwind CSS v4
- **Backend**: Rust-based Tauri runtime for native OS integration
- **Rendering**: Prerendered static site (no SSR) due to Tauri's lack of Node.js server

## Development Environment

### Nix Flake Setup
This project uses Nix flakes for reproducible development environments. The flake provides:
- Node.js 24 with corepack (for pnpm)
- Rust toolchain (rustc, cargo, clippy, rustfmt, rust-analyzer)
- Tauri dependencies (webkitgtk, gtk3, libsoup, etc.)
- LSPs for TypeScript, JSON, and YAML
- To enter the dev environment: `nix develop`

### Package Manager
Use `pnpm` for all Node.js package management (configured via corepack).

## Common Commands

### Development
- `make dev` or `pnpm tauri dev` - Start Tauri app in development mode (runs frontend dev server on port 1420 + Rust backend)
- `pnpm dev` - Run frontend dev server only (without Tauri wrapper)

### Building
- `pnpm build` - Build SvelteKit frontend for production
- `pnpm tauri build` - Build complete Tauri application (bundles frontend + Rust backend)

### Type Checking & Linting
- `pnpm check` - Run svelte-check for type errors
- `pnpm check:watch` - Run svelte-check in watch mode
- `cargo clippy` - Lint Rust code (run from `src-tauri/`)
- `cargo fmt` - Format Rust code (run from `src-tauri/`)

## Architecture Details

### Frontend (SvelteKit)
- **Adapter**: `@sveltejs/adapter-static` - prerendering only, no SSR (`ssr: false` in `src/routes/+layout.ts`)
- **Build Output**: `build/` directory (referenced by `tauri.conf.json` as `frontendDist`)
- **Routing**: File-based routing in `src/routes/`
- **Styling**: Tailwind CSS v4 with Vite plugin (`@tailwindcss/vite`)

### Backend (Tauri/Rust)
- **Entry Point**: `src-tauri/src/main.rs` calls `meeru_lib::run()` from `src-tauri/src/lib.rs`
- **Crate Type**: Hybrid library (`staticlib`, `cdylib`, `rlib`) named `meeru_lib`
- **Commands**: Rust functions exposed to frontend via `#[tauri::command]` macro (e.g., `greet`)
- **Plugins**: `tauri-plugin-opener` for system default app opening
- **IPC**: Frontend calls Rust using `@tauri-apps/api`

### Vite Configuration
- **Port**: Fixed at 1420 (strict mode) for Tauri integration
- **HMR**: Configurable via `TAURI_DEV_HOST` environment variable
- **Watch**: Ignores `src-tauri/**` to prevent circular rebuilds

### Tauri Configuration (`src-tauri/tauri.conf.json`)
- **Dev Command**: `pnpm dev` (runs before Tauri dev server)
- **Build Command**: `pnpm build` (runs before Tauri bundling)
- **Frontend URL**: `http://localhost:1420` in dev mode
- **Window**: Default 800x600, title "meeru"

## Project Structure

```
meeru/
├── src/                    # SvelteKit frontend
│   └── routes/            # File-based routing
│       ├── +layout.ts     # SSR disabled, prerender enabled
│       └── +page.svelte   # Main page component
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Binary entry point
│   │   └── lib.rs         # Library with Tauri commands
│   ├── Cargo.toml         # Rust dependencies
│   ├── tauri.conf.json    # Tauri configuration
│   └── capabilities/      # Tauri security capabilities
├── build/                 # SvelteKit build output (gitignored)
├── flake.nix             # Nix development environment
└── package.json          # Node.js dependencies and scripts
```

## Adding Features

### New Tauri Command (Rust → Frontend)
1. Add function with `#[tauri::command]` in `src-tauri/src/lib.rs`
2. Register in `.invoke_handler(tauri::generate_handler![...])` in `lib.rs`
3. Call from frontend using `invoke()` from `@tauri-apps/api/core`

### New Frontend Route
1. Create file in `src/routes/` (e.g., `+page.svelte` for route, `+page.ts` for data loading)
2. Ensure `export const prerender = true` if route-specific rendering differs from layout

### Styling Changes
- Tailwind CSS v4 uses Vite plugin - no separate config file needed
- Global styles can be added via `@theme` in CSS files (see Tailwind v4 docs)
