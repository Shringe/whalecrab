{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-25.05";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "whalecrab";
          version = "0.4.0";

          cargoLock.lockFile = ./Cargo.lock;
          src = pkgs.lib.cleanSource self;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            git
            cargo
            rustfmt
          ];
        };
      }
    );
}
