{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz") {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "zos-server";
  version = "1.0.0";

  src = pkgs.lib.cleanSource /home/mdupont/zos-server;

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
  };

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    openssl
  ];

  # Copy www directory to output
  postInstall = ''
    mkdir -p $out/www
    cp -r ${src}/www/* $out/www/ || true

    # Create metadata with store path hash
    cat > $out/metadata.json << EOF
{
  "name": "${pname}",
  "version": "${version}",
  "store_path": "$out",
  "hash": "$(basename $out | cut -d'-' -f1)"
}
EOF
  '';

  meta = with pkgs.lib; {
    description = "ZOS Server with content-addressed plugin system";
    license = licenses.gpl3;
  };
}
