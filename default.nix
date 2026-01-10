{ pkgs ? import <nixpkgs> {} }:

pkgs.stdenv.mkDerivation {
  name = "zos-server";

  buildInputs = with pkgs; [
    rustc
    cargo
    pkg-config
    openssl
    git
  ];

  src = ./.;

  buildPhase = ''
    cargo build --release --bin zos-minimal-server
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp target/release/zos-minimal-server $out/bin/
  '';
}
