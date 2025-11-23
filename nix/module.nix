{ self }:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.peer-practice;
  defaultPackage = self.packages.${pkgs.system}.app-native;

  serverConfig = {
    version = "V2025_11_17";
    email = {
      inherit (cfg.email)
        from
        reply_to
        tls_relay
        credential_email_account
        ;
      # If the password is set in nix, use it. Otherwise rely on runtime env/file replacement if your app supports it.
      # Assuming your app reads this raw from TOML:
      password = cfg.email.password;
    };
    server = {
      # Dynamically point to the package's dist folder
      webroot = "${cfg.package}/dist";
      inherit (cfg)
        jwt_secret
        data_dir
        port
        cors_allowed_origins
        ;
    };
  };

  tomlFormat = pkgs.formats.toml { };
  configFile = tomlFormat.generate "peer-practice-config.toml" serverConfig;
in
{
  options.services.peer-practice = {
    enable = lib.mkEnableOption "peer-practice service";

    package = lib.mkOption {
      type = lib.types.package;
      default = defaultPackage;
      description = "The combined peer_practice package to use.";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 3000;
      description = "Port to listen on.";
    };

    data_dir = lib.mkOption {
      type = lib.types.path;
      default = "/var/lib/peer_practice";
      description = "Directory to store application data.";
    };

    jwt_secret = lib.mkOption {
      type = lib.types.str;
      description = "Secret for JWT generation.";
    };

    cors_allowed_origins = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ "http://localhost:${toString cfg.port}" ];
      description = "List of allowed CORS origins.";
    };

    email = {
      from = lib.mkOption {
        type = lib.types.str;
        default = "noreply@example.com";
      };
      reply_to = lib.mkOption {
        type = lib.types.str;
        default = "noreply@example.com";
      };
      tls_relay = lib.mkOption {
        type = lib.types.str;
        default = "smtp.example.com";
      };
      credential_email_account = lib.mkOption {
        type = lib.types.str;
        default = "user@example.com";
      };
      password = lib.mkOption {
        type = lib.types.str;
        description = "Email password (WARNING: stored in world-readable nix store if set here)";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    # Create a system user
    users.users.peer-practice = {
      isSystemUser = true;
      group = "peer-practice";
      description = "Peer Practice service user";
      home = cfg.data_dir;
      createHome = true;
    };
    users.groups.peer-practice = { };

    systemd.services.peer-practice = {
      description = "Peer Practice Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];

      serviceConfig = {
        User = "peer-practice";
        Group = "peer-practice";
        # Point to the binary inside the package
        ExecStart = "${cfg.package}/bin/peer_practice run --config ${configFile}";
        Restart = "always";
        WorkingDirectory = cfg.data_dir;

        # Hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        # Allow writing only to data_dir
        ReadWritePaths = [ cfg.data_dir ];
      };
    };
  };
}
