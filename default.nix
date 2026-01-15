{ pkgs ? import <nixpkgs> {} }:

let
  # Get all binary names from Cargo.toml
  mainBinaries = [
    "zos_server"
    "zos-dev-server"
    "zos-dev-minimal"
    "zos-dev-launch"
  ];

  analysisBinaries = [
    "markov_1_4m_analyzer"
    "multi_repo_extractor"
    "p2p_rustc_loader"
    "p2p_rustc_test"
  ];

  allBinaries = mainBinaries ++ analysisBinaries;
in
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

  # Build all binaries
  buildPhase = ''
    cargo build --release --bins
  '';

  # Skip tests for now
  doCheck = false;

  installPhase = ''
    mkdir -p $out/bin

    # Copy all binaries
    ${pkgs.lib.concatMapStringsSep "\n    " (bin: "cp target/release/${bin} $out/bin/") allBinaries}

    # Helper scripts
    cp install-from-node.sh $out/bin/ || true
  '';

  meta = with pkgs.lib; {
    description = "ZOS Server - Zero Ontology System with resource tracing";
    license = licenses.agpl3Plus;
    maintainers = [ ];
  };
}
