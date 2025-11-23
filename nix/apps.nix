{ packages }:
{
  # Executed via `nix run`
  default = {
    type = "app";
    program = "${packages.peer-practice-configured}/bin/peer-practice-configured";
  };

  # Executed via `nix run .#native`
  native = {
    type = "app";
    program = "${packages.app-native}/bin/peer_practice";
  };
}
