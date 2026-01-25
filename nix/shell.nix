{ pkgs }:
let
  nightlyToolchain = pkgs.rust-bin.nightly.latest.default;
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    bash
    fish
    nightlyToolchain
    nodejs_24
    trunk
    wasm-bindgen-cli
    openssl.dev
    pkg-config
    cargo-nextest
    cargo-fuzz
  ];
}
