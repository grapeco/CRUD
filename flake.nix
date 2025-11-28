{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { nixpkgs, fenix, flake-utils, ... }:
  flake-utils.lib.eachDefaultSystem (system:
  let
    overlays = [ fenix.overlays.default ];
    pkgs = import nixpkgs { inherit system overlays; };
  in {
    nixpkgs.overlays = [ fenix.overlays.default ];
    devShells.default = import ./shell.nix { inherit pkgs; };
  });
}
