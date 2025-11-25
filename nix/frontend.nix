{
  pkgs,
  crane,
  root,
}:
let
  lib = pkgs.lib;
  craneLib = crane.mkLib pkgs;

  # Extract version information from the specific package Cargo.toml
  crateDetails = craneLib.crateNameFromCargoToml {
    cargoToml = root + /web-leptos/Cargo.toml;
  };
  src = lib.fileset.toSource {
    inherit root;
    fileset = lib.fileset.unions [
      (craneLib.fileset.commonCargoSources root)
      (lib.fileset.fileFilter (
        file:
        lib.any file.hasExt [
          "html"
          "scss"
          "css"
        ]
      ) root)
      (lib.fileset.maybeMissing (root + "/assets"))
    ];
  };

  commonArgs = {
    inherit src;
  };
  # Web frontend package using Trunk (wasm32 target)
  wasmToolchain = pkgs.rust-bin.stable.latest.default.override {
    targets = [ "wasm32-unknown-unknown" ];
  };

  wasmArgs = commonArgs // {
    pname = "trunk-workspace-wasm";
    version = crateDetails.version;
    cargoExtraArgs = "--package=web-leptos";
    CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
  };

  cargoArtifactsWasm = craneLib.buildDepsOnly (
    wasmArgs
    // {
      doCheck = false;
      nativeBuildInputs = [
        wasmToolchain
        pkgs.lld
      ];
    }
  );
in
# Use crane's dedicated Trunk builder for workspace-based apps
craneLib.buildTrunkPackage {
  pname = "web-leptos";
  version = crateDetails.version;
  inherit src;

  cargoArtifacts = cargoArtifactsWasm;
  # Trunk expects the current directory to be the crate to compile
  preBuild = ''
    cd ./web-leptos
  '';
  # After building, move the `dist` artifacts and restore the working directory
  postBuild = ''
    mv ./dist ../dist
    cd ..
  '';
  nativeBuildInputs = [
    wasmToolchain
    pkgs.lld
  ];
  # The version of wasm-bindgen-cli here must match the one from Cargo.lock.
  wasm-bindgen-cli = pkgs.buildWasmBindgenCli rec {
    src = pkgs.fetchCrate {
      pname = "wasm-bindgen-cli";
      version = "0.2.105";
      hash = "sha256-zLPFFgnqAWq5R2KkaTGAYqVQswfBEYm9x3OPjx8DJRY=";
    };

    cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
      inherit src;
      inherit (src) pname version;
      hash = "sha256-a2X9bzwnMWNt0fTf30qAiJ4noal/ET1jEtf5fBFj5OU=";
    };
  };
}
