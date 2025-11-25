{
  pkgs,
  utils,
  backend,
  frontend,
  root,
}:
let
  inherit (backend) peer_practice peer_practice_aarch64 peer_practice_x86_64;
  web_leptos = frontend;

  # Read version from the top-level Cargo.toml (workspace)
  workspaceToml = builtins.fromTOML (builtins.readFile (root + /Cargo.toml));
  workspaceVersion = workspaceToml.workspace.package.version;

  app-native = utils.makeCombinedApp {
    serverPkg = peer_practice;
    webPkg = web_leptos;
    pname = "peer-practice-base";
    version = workspaceVersion;
  };

  configured = utils.makeConfiguredApp {
    pname = "peer-practice-configured";
    combinedApp = app-native;
    version = workspaceVersion;
  };
in
{
  app-native = app-native;
  peer-practice-configured = configured;

  inherit peer_practice web_leptos;
  "peer_practice-aarch64" = peer_practice_aarch64;
  "peer_practice-x86_64" = peer_practice_x86_64;
}
