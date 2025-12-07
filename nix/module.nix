{ self }:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.peer-practice;
  defaultPackage = self.packages.${pkgs.stdenv.hostPlatform.system}.app-native;

  serverConfig = {
    version = "V2025_11_23";

    email = {
      inherit (cfg.email)
        from
        reply_to
        tls_relay
        credential_email_account
        ;
      # Write password_file instead of an inline password
      password_file = cfg.email.password_file;
    };

    server = {
      # Keep dynamic webroot pointing at built dist, still matches TOML key
      webroot = "${cfg.package}/dist";

      inherit (cfg)
        jwt_secret_file
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

    jwt_secret_file = lib.mkOption {
      type = lib.types.path;
      description = "Path to file containing the JWT secret.";
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
      password_file = lib.mkOption {
        type = lib.types.path;
        default = "/var/peer_practice/email-password.txt";
        description = "Path to file containing the email account password.";
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
