{
  description = "ZOS Cross-Platform Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rust-bin.nightly.latest.default.override {
              extensions = [ "rust-src" "rustfmt" "clippy" ];
              targets = [
                "x86_64-unknown-linux-gnu"
                "x86_64-pc-windows-gnu"
                "aarch64-unknown-linux-gnu"
                "aarch64-linux-android"
                "armv7-linux-androideabi"
              ];
            })
            pkgsCross.mingwW64.stdenv.cc
            pkgsCross.aarch64-multiplatform.stdenv.cc
            pkg-config
            openssl
            openssl.dev
            curl
            git
            gcc
            libiconv
            zlib
            zlib.dev
          ];

          shellHook = ''
            export NIXPKGS_ACCEPT_ANDROID_SDK_LICENSE=1
            export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/x86_64-w64-mingw32-gcc
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=${pkgs.pkgsCross.aarch64-multiplatform.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc
            export OPENSSL_DIR=${pkgs.openssl.dev}
            export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
            export OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include
            export PKG_CONFIG_PATH=${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH
            echo "🚀 ZOS Cross-Platform Dev Environment Ready"
            echo "📱 Run: ./build-android.sh"
          '';
        };
      });
}
