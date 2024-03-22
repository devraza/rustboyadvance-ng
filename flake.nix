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
    ...
  }:
    utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs-unstable {
          inherit system;
          overlays = [fenix.overlays.default];
        };
        toolchain = pkgs.fenix.complete;
        dependencies = with pkgs; [ SDL2 SDL2_image ];
      in rec
      {
        # Executed by `nix build`
        packages.default =
          (pkgs.makeRustPlatform {
            inherit (toolchain) cargo rustc;
          })
          .buildRustPackage {
            pname = "rustboyadvance-ng";
            name = "rustboyadvance-ng";
            src = ./.;
            meta.mainProgram = "rustboyadvance-sdl2";
            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = {
                "libretro-backend-0.2.1" = "sha256-qsJo7wP01zhRNv4XrZJbIvOQrSJfUaqve0fNOaR6aWs=";
              };
            };
            buildInputs = dependencies;
          };

        # Executed by `nix run`
        apps.default = utils.lib.mkApp {drv = packages.default;};

        # Used by `nix develop`
        devShells.default = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            (with toolchain; [
              cargo rustc rust-src clippy rustfmt # rust components
            ])
          ] ++ dependencies;
          RUST_SRC_PATH = "${toolchain.rust-src}/lib/rustlib/src/rust/library";
        };
      }
    );
}
