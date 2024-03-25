{
  description = "Rust development environment for oxitoko using fenix";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs-unstable,
    utils,
    ...
  }:
    utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs-unstable { inherit system; };
        toolchain = pkgs.fenix.complete;
        dependencies = with pkgs; [ SDL2 SDL2_image ];
      in rec
      {
        # Executed by `nix build`
        packages.default =
          (pkgs.makeRustPlatform.buildRustPackage {
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
        devShells.default = pkgs.mkShell rec { buildInputs = dependencies; };
      }
    );
}
