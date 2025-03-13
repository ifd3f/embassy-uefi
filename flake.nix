{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, flake-utils, nixpkgs, rust-overlay, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        pkgsCrossEmbedded = pkgs.pkgsCross.x86_64-embedded;
        rustToolchain =
          pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        naerskLib = pkgs.callPackage naersk {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
      in rec {
        # expose these fields for debugging nix issues
        debug = { inherit pkgs pkgsCrossEmbedded naerskLib rustToolchain; };

        devShells.default = with pkgs;
          mkShell {
            buildInputs =
              [ mtools rustToolchain pre-commit pkgsCrossEmbedded.stdenv.cc ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      });

}
