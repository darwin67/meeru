{
  description = "Meeru - Cross Platform Email App";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
        corepack = pkgs.stdenv.mkDerivation {
          name = "corepack";
          buildInputs = [ pkgs.nodejs_24 ];
          phases = [ "installPhase" ];
          installPhase = ''
            mkdir -p $out/bin
            corepack enable --install-directory=$out/bin
          '';
        };

      in {
        devShells.default = pkgs.mkShell {
          packages = [ corepack ];

          buildInputs = with pkgs; [
            rustc
            rustup
            rustfmt
            cargo
            clippy

            # deps
            cargo-tauri
            pkg-config
            openssl
            # webkitgtk_6_0
            gtk3
            librsvg
            libsoup_3
            atkmm
            at-spi2-atk
            webkitgtk_4_1

            # Node
            typescript
            nodejs_24

            # LSPs
            rust-analyzer
            nodePackages.typescript-language-server
            nodePackages.vscode-json-languageserver
            nodePackages.yaml-language-server

            # Tools
            git-cliff
            claude-code
          ];

          RUST_SRC_PATH =
            "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      });
}
