{
  pkgs,
  crane,
  root,
}:
let
  craneLib = crane.mkLib pkgs;
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  # Re-use the same source definition as backend to maximize caching
  src = pkgs.lib.fileset.toSource {
    inherit root;
    fileset = craneLib.fileset.commonCargoSources root;
  };

  commonArgs = {
    inherit src;
    cargoExtraArgs = "-p peer_practice --locked";
    buildInputs = [
      pkgs.openssl
    ]
    ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
      pkgs.libiconv
      pkgs.darwin.apple_sdk.frameworks.Security
    ];
  };
in
{
  # Run tests using cargo-nextest
  peer_practice_test = craneLib.cargoNextest (
    commonArgs
    // {
      inherit cargoArtifacts;
      partitions = 1;
      partitionType = "count";
    }
  );

  # Run clippy linting
  peer_practice_clippy = craneLib.cargoClippy (
    commonArgs
    // {
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "--all-targets -- --deny warnings";
    }
  );

  # Check formatting
  peer_practice_fmt = craneLib.cargoFmt (commonArgs) // {
    inherit cargoArtifacts;
    inherit src;
  };
}
