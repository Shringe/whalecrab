{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    let
      systemOutputs = inputs.flake-utils.lib.eachDefaultSystem (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          version = cargoToml.workspace.package.version;
        in
        {
          packages = {
            tui = pkgs.rustPlatform.buildRustPackage {
              inherit version;
              name = "tui";
              pname = "whalecrab_tui";

              cargoLock.lockFile = ./Cargo.lock;
              src = self;
            };

            uci = pkgs.rustPlatform.buildRustPackage {
              inherit version;
              name = "uci";
              pname = "whalecrab_uci";

              cargoLock.lockFile = ./Cargo.lock;
              src = self;
            };
          };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              git
              rustc
              rustfmt
              cargo
              rust-analyzer
              gnuplot
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
