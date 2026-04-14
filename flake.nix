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
            name: pname: profile:
            pkgs.rustPlatform.buildRustPackage {
              inherit version name pname;
              src = self;
              cargoLock.lockFile = ./Cargo.lock;

              # It is highly recommended to build with native optimizations
              RUSTFLAGS = "-C target-cpu=native";
              buildType = profile;

              # Ensure only the necessary dependencies get built
              cargoTestFlags = [
                "--package"
                name
              ];
              cargoBuildFlags = [
                "--package"
                name
              ];
            };
        in
        {
          packages = {
            tui = makeCratePackage "tui" "whalecrab_tui" "release";
            uci = makeCratePackage "uci" "whalecrab_uci" "release";
            tui_canary = makeCratePackage "tui" "whalecrab_tui" "canary";
            uci_canary = makeCratePackage "uci" "whalecrab_uci" "canary";
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
