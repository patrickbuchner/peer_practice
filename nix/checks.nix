{
  pkgs,
  crane,
  root,
}:
let
  craneLib = crane.mkLib pkgs;

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
      partitions = 1;
      partitionType = "count";
    }
  );

  # Run clippy linting
  peer_practice_clippy = craneLib.cargoClippy (
    commonArgs
    // {
      cargoClippyExtraArgs = "--all-targets -- --deny warnings";
    }
  );

  # Check formatting
  peer_practice_fmt = craneLib.cargoFmt {
    inherit src;
  };
}
