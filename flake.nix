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
            libsecret
          ];

          shellHook = ''
            export PATH="$HOME/.pub-cache/bin:$PATH"
          '';
        };
      });
}
