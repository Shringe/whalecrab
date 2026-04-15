{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    let
      systemOutputs = flake-utils.lib.eachDefaultSystem (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          version = cargoToml.workspace.package.version;

          makeCratePackage =
            name: pname: extraAttrs:
            pkgs.rustPlatform.buildRustPackage (
              {
                inherit version name pname;
                src = self;
                cargoLock.lockFile = ./Cargo.lock;

                # It is highly recommended to build with native optimizations
                RUSTFLAGS = "-C target-cpu=native";

                # Ensure only the necessary dependencies get built
                cargoTestFlags = [
                  "--package"
                  name
                ];
                cargoBuildFlags = [
                  "--package"
                  name
                ];
              }
              // extraAttrs
            );

          makeCratePackageCanary =
            name: pname:
            makeCratePackage name pname {
              buildType = "canary";
              buildNoDefaultFeatures = true;
              buildFeatures = [
                "panic_logger"
              ];
            };
        in
        {
          packages = {
            tui = makeCratePackage "tui" "whalecrab_tui" { };
            uci = makeCratePackage "uci" "whalecrab_uci" { };
            tui_canary = makeCratePackageCanary "tui" "whalecrab_tui";
            uci_canary = makeCratePackageCanary "uci" "whalecrab_uci";
          };

          devShells.default = pkgs.mkShell {
            shellHook = ''
              export RUSTFLAGS="-C target-cpu=native"
            '';

            buildInputs = with pkgs; [
              git
              rustc
              rustfmt
              cargo
              rust-analyzer
              gnuplot
              clippy
              cutechess
            ];
          };
        }
      );
    in
    systemOutputs
    // {
      overlay = final: prev: {
        whalecrab_tui = systemOutputs.packages.${prev.system}.tui;
        whalecrab_uci = systemOutputs.packages.${prev.system}.uci;
        whalecrab_tui_canary = systemOutputs.packages.${prev.system}.tui_canary;
        whalecrab_uci_canary = systemOutputs.packages.${prev.system}.uci_canary;
      };
    };
}
