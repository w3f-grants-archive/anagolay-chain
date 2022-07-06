# NOT WORKING 
let
  mozillaOverlay =
    import (builtins.fetchGit {
      url = "https://github.com/mozilla/nixpkgs-mozilla.git";
      rev = "4a07484cf0e49047f82d83fd119acffbad3b235f";
    });
  nixpkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  rust-nightly = with nixpkgs; ((rustChannelOf { date = "2021-10-14"; channel = "nightly"; }).rust.override {
    extensions = [ "rust-src" ];
    targets = [ "wasm32-unknown-unknown" ];
  });
in

with nixpkgs;
pkgs.mkShell {
  buildInputs = [
    figlet
    nodejs-18_x
    nodePackages.pnpm
    neovim
    git
    clang
    openssl.dev
    pkg-config
    rust-nightly
    wasm-pack
  ];

  # env variables
  RUST_SRC_PATH = "${rust-nightly}/src/rustlib/src/rust/src";
  LIBCLANG_PATH = "${llvmPackages.libclang.src}/src";
  PROTOC = "${protobuf}/bin/protoc";
  ROCKSDB_LIB_DIR = "${rocksdb}/src";
  # env variables

  shellHook = ''
    alias ll="ls -l"
    alias la="ls -la"
    alias lah="ls -lah"
      echo "Anagolay Chain Env" | figlet -w 100%
  '';
}

