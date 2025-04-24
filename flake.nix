{
  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/release-24.11";
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
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            rust-bin.beta.latest.default
            fish
          ];

          shellHook = ''
            exec fish
          '';
        };
      }
    );
}
