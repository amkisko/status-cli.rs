# Nix flake for status-cli. Run from repo root: nix build .#default
{
  description = "Status page CLI — search catalog and check live status";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "status-cli";
          version = "0.1.0";
          src = self;
          cargoLock.lockFile = self + "/Cargo.lock";
          buildAndTestSubdir = "status";
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/status";
        };
      });
}
