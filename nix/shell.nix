{ pkgs }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    bash
    fish
    rustup
    nodejs_24
    trunk
    wasm-bindgen-cli
    openssl.dev
    pkg-config
    cargo-nextest
  ];
}
