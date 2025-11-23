{
  pkgs,
  crane,
  root,
}:
let
  lib = pkgs.lib;
  craneLib = crane.mkLib pkgs;

  src = lib.fileset.toSource {
    inherit root;
    fileset = craneLib.fileset.commonCargoSources root;
  };

  commonArgs = {
    inherit src;
    cargoExtraArgs = "-p peer_practice --locked";
  };

  # Native server package
  peer_practice = craneLib.buildPackage (
    commonArgs
    // {
      pname = "peer_practice";
      version = "0.1.0";
    }
  );

  # Define explicit Rust toolchains that include the required std components for targets
  rustToolchainAarch64 = pkgs.rust-bin.stable.latest.default.override {
    targets = [ "aarch64-unknown-linux-musl" ];
  };
  rustToolchainX86_64 = pkgs.rust-bin.stable.latest.default.override {
    targets = [ "x86_64-unknown-linux-gnu" ];
  };

  # Helper to produce a cross-compiled server package for a given target
  makeTargetPkg =
    target: crossPkgs: rustToolchain:
    let
      targetEnv = builtins.replaceStrings [ "-" ] [ "_" ] (lib.strings.toUpper target);
      targetEnvLower = builtins.replaceStrings [ "-" ] [ "_" ] target;
      linkerVar = "CARGO_TARGET_" + targetEnv + "_LINKER";
    in
    craneLib.buildPackage (
      (
        commonArgs
        // {
          pname = "peer_practice";
          version = "0.0.0";
          CARGO_BUILD_TARGET = target;
          "${linkerVar}" = "${crossPkgs.stdenv.cc.targetPrefix}cc";
          "CC_${targetEnv}" = "${crossPkgs.stdenv.cc.targetPrefix}cc";
          "AR_${targetEnv}" = "${crossPkgs.stdenv.cc.bintools.targetPrefix}ar";
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

  # Cross toolchain targeting aarch64 with MUSL libc and x86_64
  peer_practice_aarch64 =
    makeTargetPkg "aarch64-unknown-linux-musl" pkgs.pkgsCross.aarch64-multiplatform-musl
      rustToolchainAarch64;
  peer_practice_x86_64 =
    makeTargetPkg "x86_64-unknown-linux-gnu" pkgs.pkgsCross.x86_64-multiplatform
      rustToolchainX86_64;
in
{
  inherit peer_practice peer_practice_aarch64 peer_practice_x86_64;
}
