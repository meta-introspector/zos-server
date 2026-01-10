let
  oxalica-overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  nixpkgs = import <nixpkgs> { overlays = [ oxalica-overlay ]; };
in
nixpkgs.mkShell {
  buildInputs = with nixpkgs; [
    # Nightly Rust with cross-compilation targets
    (rust-bin.nightly.latest.default.override {
      extensions = [ "rust-src" "rustfmt" "clippy" ];
      targets = [
        "x86_64-unknown-linux-gnu"    # Linux x64 (host)
        "x86_64-pc-windows-gnu"       # Windows x64
        "aarch64-unknown-linux-gnu"   # ARM64 Linux (OCI)
        "aarch64-linux-android"       # Android ARM64
        "armv7-linux-androideabi"     # Android ARM32
      ];
    })

    # Cross-compilation toolchains
    pkgsCross.mingwW64.stdenv.cc      # Windows
    pkgsCross.aarch64-multiplatform.stdenv.cc  # ARM64 Linux

    # Build tools
    pkg-config
    openssl
    git
    systemd

    # Cross-compilation libraries
    pkgsCross.aarch64-multiplatform.openssl
    pkgsCross.aarch64-multiplatform.pkg-config
    pkgsCross.mingwW64.openssl
    pkgsCross.mingwW64.pkg-config

    # Windows cross-compilation
    wineWowPackages.stable
  ];

  RUST_SRC_PATH = "${nixpkgs.rust-bin.nightly.latest.default}/lib/rustlib/src/rust/library";

  # Cross-compilation environment variables
  CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${nixpkgs.pkgsCross.mingwW64.stdenv.cc}/bin/x86_64-w64-mingw32-gcc";
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${nixpkgs.pkgsCross.aarch64-multiplatform.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc";

  # OpenSSL for cross-compilation - ARM64
  AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_DIR = "${nixpkgs.pkgsCross.aarch64-multiplatform.openssl.dev}";
  AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR = "${nixpkgs.pkgsCross.aarch64-multiplatform.openssl.out}/lib";
  AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR = "${nixpkgs.pkgsCross.aarch64-multiplatform.openssl.dev}/include";

  # OpenSSL for cross-compilation - Windows
  X86_64_PC_WINDOWS_GNU_OPENSSL_DIR = "${nixpkgs.pkgsCross.mingwW64.openssl.dev}";
  X86_64_PC_WINDOWS_GNU_OPENSSL_LIB_DIR = "${nixpkgs.pkgsCross.mingwW64.openssl.out}/lib";
  X86_64_PC_WINDOWS_GNU_OPENSSL_INCLUDE_DIR = "${nixpkgs.pkgsCross.mingwW64.openssl.dev}/include";

  # Host OpenSSL
  OPENSSL_DIR = "${nixpkgs.openssl.dev}";
  OPENSSL_LIB_DIR = "${nixpkgs.openssl.out}/lib";
  OPENSSL_INCLUDE_DIR = "${nixpkgs.openssl.dev}/include";

  # PKG_CONFIG for cross-compilation
  PKG_CONFIG_ALLOW_CROSS = "1";
  PKG_CONFIG_AARCH64_UNKNOWN_LINUX_GNU = "${nixpkgs.pkgsCross.aarch64-multiplatform.pkg-config}/bin/aarch64-unknown-linux-gnu-pkg-config";
  PKG_CONFIG_X86_64_PC_WINDOWS_GNU = "${nixpkgs.pkgsCross.mingwW64.pkg-config}/bin/x86_64-w64-mingw32-pkg-config";
}
