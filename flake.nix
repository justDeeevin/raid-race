{
  # TODO:
  # description = "";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
        );

        src = craneLib.cleanCargoSource ./.;

        dylibs = with pkgs; [
          libxkbcommon
          vulkan-loader
          wayland
        ];

        # Common arguments can be set here to avoid repeating them later
        # Note: changes here will rebuild all dependency crates
        commonArgs = {
          inherit src;
          strictDeps = true;
          nativeBuildInputs = [ pkgs.pkg-config ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
        };

        fileSetForCrate =
          crate:
          lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              (craneLib.fileset.commonCargoSources ./crates/hakari)
              (craneLib.fileset.commonCargoSources ./crates/lib)
              (craneLib.fileset.commonCargoSources crate)
            ];
          };

        raid-race = craneLib.buildPackage (
          individualCrateArgs
          // rec {
            pname = "raid-race";
            cargoExtraArgs = "-p ${pname}";
            src = fileSetForCrate ./crates/game;

            nativeBuildInputs = with pkgs; [
              autoPatchelfHook
            ];

            buildInputs =
              with pkgs;
              [
                kdePackages.wayland.dev
                alsa-lib.dev
                libudev-zero
              ]
              ++ dylibs;
          }
        );
        raid-race-server = craneLib.buildPackage (
          individualCrateArgs
          // rec {
            pname = "raid-race-server";
            cargoExtraArgs = "-p ${pname}";
            src = fileSetForCrate ./crates/server;

            buildInputs = [ pkgs.openssl.dev ];
          }
        );
      in
      {
        checks = {
          inherit raid-race raid-race-server;
          hakari = craneLib.mkCargoDerivation {
            inherit src;
            pname = "hakari";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;
            nativeBuildInputs = [ pkgs.cargo-hakari ];

            buildPhaseCargoCommand =
              # bash
              ''
                cargo hakari generate --diff
                cargo hakari manage-deps --dry-run
                cargo hakari verify
              '';
          };
        };

        packages = { inherit raid-race raid-race-server; };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          env.LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath dylibs;
        };
      }
    );
}
