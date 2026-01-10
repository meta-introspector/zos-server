{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "zos-server";
  version = "1.0.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    openssl
  ];

  # Build the standalone server
  buildPhase = ''
    cd zos-minimal-server
    cargo build --release
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp target/release/zos-minimal-server $out/bin/
    cp ../install-from-node.sh $out/bin/
  '';

  meta = with pkgs.lib; {
    description = "ZOS Server - Zero Ontology System with resource tracing";
    license = licenses.agpl3Plus;
    maintainers = [ ];
  };
}
