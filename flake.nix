{
  description = "Rust development environment for oxitoko using fenix";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs-unstable";
    };
    utils.url = "github:numtide/flake-utils";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs-unstable,
    utils,
    fenix,
    nixgl,
    ...
  }:
    utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs-unstable {
          inherit system;
          overlays = [fenix.overlays.default nixgl.overlay];
        };
        toolchain = pkgs.fenix.complete;
      in rec
      {
        # Executed by `nix build`
        packages.default =
          (pkgs.makeRustPlatform {
            inherit (toolchain) cargo rustc;
          })
          .buildRustPackage {
            pname = "oxitoko";
            version = "1.0.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };

        # Executed by `nix run`
        apps.default = utils.lib.mkApp {drv = packages.default;};

        # Used by `nix develop`
        devShells.default = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            (with toolchain; [
              cargo rustc rust-src clippy rustfmt # rust components
            ])
            SDL2 SDL2_image
            mold clang
          ];
          RUST_SRC_PATH = "${toolchain.rust-src}/lib/rustlib/src/rust/library";
        };
      }
    );
}
