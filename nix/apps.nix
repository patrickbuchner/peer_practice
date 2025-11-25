{ packages }:
{
  # Executed via `nix run`
  default = {
    type = "app";
    program = "${packages.app-native}/bin/peer_practice";
  };
}
