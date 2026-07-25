{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage {
  pname = "status-cli";
  version = "0.2.0";
  src = ./../..;
  cargoLock.lockFile = ./../../Cargo.lock;
  buildAndTestSubdir = "status";
}
