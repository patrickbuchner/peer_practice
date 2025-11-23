{ pkgs }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    bash
    fish
    rustup
    nodejs_20
    trunk
    wasm-bindgen-cli
    openssl.dev
    pkg-config
    cargo-nextest
  ];
}
