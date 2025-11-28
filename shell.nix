{ pkgs ? import <nixpkgs> {}, }:

with pkgs;

mkShell {
  nativeBuildInputs = [ pkg-config ];
  buildInputs = with pkgs; [
    (pkgs.fenix.complete.withComponents [
      "cargo"
      "rustc"
      "rust-src"
    ])
    openssl
    sqlx-cli
  ];

  LD_LIBRARY_PATH = lib.makeLibraryPath [ openssl ];

  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
  '';
}