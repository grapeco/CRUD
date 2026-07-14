{ pkgs ? import <nixpkgs> {}, }:

with pkgs;

mkShell rec {
  nativeBuildInputs = [ pkg-config ];
  buildInputs = with pkgs; [
    # (with pkgs.fenix; combine [
    #   complete.toolchain
    #   targets.x86_64-unknown-linux-musl.latest.rust-std
    # ])
    rustc
    cargo
    openssl
    sqlx-cli
  ];

  LD_LIBRARY_PATH = lib.makeLibraryPath [ buildInputs ];

  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
  '';
}