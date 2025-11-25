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

        utils = import ./nix/utils.nix { inherit pkgs; };

        backend = import ./nix/backend.nix {
          inherit pkgs crane;
          root = ./.;
        };

        frontend = import ./nix/frontend.nix {
          inherit pkgs crane;
          root = ./.;
        };

        myPackages = import ./nix/packages.nix {
          inherit
            pkgs
            utils
            backend
            frontend
            ;
          root = ./.;
        };
      in
      {
        apps = import ./nix/apps.nix {
          packages = myPackages;
        };

        checks = import ./nix/checks.nix {
          inherit pkgs crane;
          root = ./.;
        };

        packages = import ./nix/packages.nix {
          inherit
            pkgs
            utils
            backend
            frontend
            ;
          root = ./.;
        };

        devShells.default = import ./nix/shell.nix { inherit pkgs; };
      }
    )
    // {
      nixosModules.default = import ./nix/module.nix { inherit self; };
    };
}
