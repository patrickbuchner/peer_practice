{
  pkgs,
  utils,
  backend,
  frontend,
}:
let
  inherit (backend) peer_practice peer_practice_aarch64 peer_practice_x86_64;
  web_leptos = frontend;
  app-native = utils.makeCombinedApp {
    serverPkg = peer_practice;
    webPkg = web_leptos;
    pname = "peer-practice-base";
  };
in
{
  inherit app-native;
  peer-practice-configured = utils.makeConfiguredApp {
    pname = "peer-practice-configured";
    combinedApp = app-native;
  };

  inherit peer_practice web_leptos;
  "peer_practice-aarch64" = peer_practice_aarch64;
  "peer_practice-x86_64" = peer_practice_x86_64;
}
