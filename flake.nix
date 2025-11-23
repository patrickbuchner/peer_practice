{
  description = "Build peer_practice (server) and web-leptos (frontend) with crane; provide cross targets";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
      rust-overlay,
    }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ] (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        lib = pkgs.lib;
        craneLib = crane.mkLib pkgs;

        # Source for the whole workspace
        unfilteredRoot = ./.; # The original, unfiltered source
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (lib.fileset.fileFilter (
              file:
              lib.any file.hasExt [
                "html"
                "scss"
                "css"
              ]
            ) unfilteredRoot)
            # Example of a folder for images, icons, etc
            (lib.fileset.maybeMissing ./assets)
          ];
        };

        # Build only the main server binary from the workspace
        commonArgs = {
          inherit src;
          cargoExtraArgs = "-p peer_practice --locked";
        };

        # Native server package
        peer_practice = craneLib.buildPackage (
          commonArgs
          // {
            pname = "peer_practice";
            version = "0.0.0";
          }
        );

        # Define explicit Rust toolchains that include the required std components for targets
        # Use MUSL for aarch64 cross-compilation
        rustToolchainAarch64 = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "aarch64-unknown-linux-musl" ];
        };
        rustToolchainX86_64 = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "x86_64-unknown-linux-gnu" ];
        };

        # Helper to produce a cross-compiled server package for a given target, with a toolchain that includes that target
        makeTargetPkg =
          target: crossPkgs: rustToolchain:
          let
            targetEnv = builtins.replaceStrings [ "-" ] [ "_" ] (lib.strings.toUpper target);
            targetEnvLower = builtins.replaceStrings [ "-" ] [ "_" ] target;
            linkerVar = "CARGO_TARGET_" + targetEnv + "_LINKER";
          in
          # Use the provided toolchain (with target std installed) by putting it in PATH
          craneLib.buildPackage (
            (
              commonArgs
              // {
                pname = "peer_practice";
                version = "0.0.0";
                CARGO_BUILD_TARGET = target;
                "${linkerVar}" = "${crossPkgs.stdenv.cc.targetPrefix}cc";
                # Ensure cc-rs and build scripts use the target C toolchain
                # e.g. CC_AARCH64_UNKNOWN_LINUX_GNU, AR_AARCH64_UNKNOWN_LINUX_GNU
                "CC_${targetEnv}" = "${crossPkgs.stdenv.cc.targetPrefix}cc";
                "AR_${targetEnv}" = "${crossPkgs.stdenv.cc.bintools.targetPrefix}ar";
                # cc crate tends to look for lowercase target variant as well
                "CC_${targetEnvLower}" = "${crossPkgs.stdenv.cc.targetPrefix}cc";
                "AR_${targetEnvLower}" = "${crossPkgs.stdenv.cc.bintools.targetPrefix}ar";
                "CARGO_TARGET_${targetEnv}_AR" = "${crossPkgs.stdenv.cc.bintools.targetPrefix}ar";
                "CARGO_TARGET_${targetEnv}_RANLIB" = "${crossPkgs.stdenv.cc.bintools.targetPrefix}ranlib";
                PKG_CONFIG_ALLOW_CROSS = "1";
                nativeBuildInputs = [
                  crossPkgs.stdenv.cc
                  pkgs.pkg-config
                  rustToolchain
                ];
                doCheck = false;
              }
            )
          );

        # Cross toolchain targeting aarch64 with MUSL libc
        crossAarch64 = pkgs.pkgsCross.aarch64-multiplatform-musl;
        crossX86_64 = pkgs.pkgsCross.x86_64-multiplatform;

        # Build for aarch64-unknown-linux-musl instead of gnu
        peer_practice_aarch64 = makeTargetPkg "aarch64-unknown-linux-musl" crossAarch64 rustToolchainAarch64;
        peer_practice_x86_64 = makeTargetPkg "x86_64-unknown-linux-gnu" crossX86_64 rustToolchainX86_64;

        # Web frontend package using Trunk (wasm32 target)
        wasmToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };

        wasmArgs = commonArgs // {
          pname = "trunk-workspace-wasm";
          version = "0.0.0";
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
        # Use crane's dedicated Trunk builder for workspace-based apps
        web_leptos = craneLib.buildTrunkPackage {
          pname = "web-leptos";
          version = "0.0.0";
          # Use the cleaned workspace src
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
          # When updating to a new version replace the hash values with lib.fakeHash,
          # then try to do a build, which will fail but will print out the correct value
          # for `hash`. Replace the value and then repeat the process but this time the
          # printed value will be for the second `hash` below
          wasm-bindgen-cli = pkgs.buildWasmBindgenCli rec {
            src = pkgs.fetchCrate {
              pname = "wasm-bindgen-cli";
              version = "0.2.105";
              hash = "sha256-zLPFFgnqAWq5R2KkaTGAYqVQswfBEYm9x3OPjx8DJRY=";
              # hash = lib.fakeHash;
            };

            cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
              inherit src;
              inherit (src) pname version;
              hash = "sha256-a2X9bzwnMWNt0fTf30qAiJ4noal/ET1jEtf5fBFj5OU=";
              # hash = lib.fakeHash;
            };
          };
        };
        makeCombinedApp =
          {
            serverPkg,
            webPkg,
            pname,
          }:
          let
            # Wrap the web package contents (webPkg) inside a 'dist' directory
            webPkgInDist =
              pkgs.runCommand "${pname}-web-dist"
                {
                  # This makes all files from webPkg available
                  nativeBuildInputs = [ webPkg ];
                }
                ''
                  mkdir -p $out/dist
                  # Link all files from webPkg's root into $out/dist
                  # The path '${webPkg}' points to the root of the web_leptos build output
                  ln -s ${webPkg}/* $out/dist/
                '';
          in
          pkgs.symlinkJoin {
            name = pname;
            paths = [
              serverPkg
              webPkgInDist # Use the new wrapped package
            ];
          };
      in
      {
        packages = {
          app-native = makeCombinedApp {
            serverPkg = peer_practice;
            webPkg = web_leptos;
            pname = "peer-practice-app-native";
          };

          app-x86_64 = makeCombinedApp {
            serverPkg = peer_practice_x86_64;
            webPkg = web_leptos;
            pname = "peer-practice-app-x86_64";
          };

          app-aarch64 = makeCombinedApp {
            serverPkg = peer_practice_aarch64;
            webPkg = web_leptos;
            pname = "peer-practice-app-aarch64";
          };

          peer_practice = peer_practice;
          "peer_practice-aarch64" = peer_practice_aarch64;
          "peer_practice-x86_64" = peer_practice_x86_64;
          web-leptos = web_leptos;
        };

        devShells.default = pkgs.mkShell {
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
        };
      }
    );
}
