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
            name: pname:
            pkgs.rustPlatform.buildRustPackage {
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
            };
        in
        {
          packages = {
            tui = makeCratePackage "tui" "whalecrab_tui";
            uci = makeCratePackage "uci" "whalecrab_uci";
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
      };
    };
}
