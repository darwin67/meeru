{
  description = "Email client";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=master";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };

      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            flutter

            # tools
            git-cliff
            claude-code
            gemini-cli

            # deps
            pkg-config
            libsecret
          ];

          shellHook = ''
            if [ -z "$PUB_CACHE" ]; then
              export PATH="$PATH:$HOME/.pub-cache/bin"
            else
              export PATH="$PATH:$PUB_CACHE/bin"
            fi

            dart pub global activate protoc_plugin
          '';

          FLUTTER_ROOT = pkgs.flutter;
          DART_ROOT = "${pkgs.flutter}/bin/cache/dart-sdk";
          # emulator related: vulkan-loader and libGL shared libs are necessary for hardware decoding
          LD_LIBRARY_PATH =
            "${pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader pkgs.libGL ]}";
          CMAKE_PREFIX_PATH =
            "${pkgs.lib.makeLibraryPath [ pkgs.libsecret.dev pkgs.gtk3.dev ]}";
        };
      });
}
