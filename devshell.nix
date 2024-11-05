{
  mkShell,
  lib,
  rust-analyzer-unwrapped,
  rustfmt,
  clippy,
  cargo,
  rustc,
  rustPlatform,
  openssl,
  pkg-config,
  libiconv,
  pkgs
}:
mkShell {
  strictDeps = true;

  nativeBuildInputs = [
    cargo
    rustc

    rust-analyzer-unwrapped
    rustfmt
    clippy
    openssl
    pkg-config
  ];

  buildInputs = [];

  packages = [
    pkgs.dlib
    pkgs.cmake
  ];

  shellHook = ''
    export PKG_CONFIG_PATH="${openssl.dev}/lib/pkgconfig";
  '';

  env = {
    RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
  };
}
