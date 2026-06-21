{
  description = "An experimental Lisp interpreter written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        # Default package built using the default.nix configuration
        packages.default = pkgs.callPackage ./default.nix {};

        # Development environment (can be entered with `nix develop`)
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
          ];

          shellHook = ''
            export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
            echo "🦀 Welcome to the Rust/Nix development environment! 🦀"
            rustc --version
          '';
        };
      }
    );
}
