{ pkgs }:
{
  makeCombinedApp =
    {
      serverPkg,
      webPkg,
      pname,
      version,
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
      name = "${pname}-${version}";
      paths = [
        serverPkg
        webPkgInDist # Use the new wrapped package
      ];
    };
}
