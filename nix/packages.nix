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
    pname = "peer_practice-base";
    version = workspaceVersion;
  };

  app_aarch64 = utils.makeCombinedApp {
    serverPkg = peer_practice_aarch64;
    webPkg = web_leptos;
    pname = "peer_practice-aarch64";
    version = workspaceVersion;
  };

  app_x86_64 = utils.makeCombinedApp {
    serverPkg = peer_practice_x86_64;
    webPkg = web_leptos;
    pname = "peer_practice-x86_64";
    version = workspaceVersion;
  };
in
{
  app-native = app-native;

  inherit peer_practice web_leptos;
  "app-aarch64-musl" = app_aarch64;
  "app-x86_64-musl" = app_x86_64;
}
