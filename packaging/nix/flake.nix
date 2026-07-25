{
  description = "Status page CLI packaging flake";

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
          version = "0.2.0";
          src = ./../..;
          cargoLock.lockFile = ./../../Cargo.lock;
          buildAndTestSubdir = "status";
        };
      });
}
