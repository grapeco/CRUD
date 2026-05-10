{ pkgs ? import <nixpkgs> {}, }:

with pkgs;

mkShell {
  nativeBuildInputs = [ pkg-config ];
  buildInputs = with pkgs; [
    (with pkgs.fenix; combine [
      complete.toolchain
      targets.x86_64-unknown-linux-musl.latest.rust-std
    ])
    openssl
    sqlx-cli
  ];

  LD_LIBRARY_PATH = lib.makeLibraryPath [ openssl ];

  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
  '';
}