
{
  description = "A basic Rust devshell";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustTarget = pkgs.rust-bin.stable.latest.default.override {
              extensions= [ "rust-src" "rust-analyzer" ];
              targets = [ "wasm32-wasi" "wasm32-unknown-unknown" ];
            };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            openssl
            pkg-config
            rustTarget
          ];

          shellHook = ''
            '';
        RUST_SRC_PATH = "${rustTarget}/lib/rustlib/src/rust/library";

        };
      }
    );
}
