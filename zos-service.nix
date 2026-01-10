{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.zos-server;
  zos-package = pkgs.callPackage ./default.nix {};
in {
  options.services.zos-server = {
    enable = mkEnableOption "ZOS Server";

    port = mkOption {
      type = types.int;
      default = 8080;
      description = "Port for ZOS server";
    };

    user = mkOption {
      type = types.str;
      default = "zos";
      description = "User to run ZOS server as";
    };

    dataDir = mkOption {
      type = types.path;
      default = "/var/lib/zos";
      description = "Data directory for ZOS server";
    };
  };

  config = mkIf cfg.enable {
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.user;
      home = cfg.dataDir;
      createHome = true;
    };

    users.groups.${cfg.user} = {};

    systemd.services.zos-server = {
      description = "ZOS Server - Zero Ontology System";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      environment = {
        ZOS_HTTP_PORT = toString cfg.port;
        ZOS_DATA_DIR = cfg.dataDir;
        ZOS_LOG_LEVEL = "info";
      };

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.user;
        WorkingDirectory = cfg.dataDir;
        ExecStart = "${zos-package}/bin/zos-minimal-server";
        Restart = "always";
        RestartSec = 5;

        # Security settings
        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ cfg.dataDir ];
      };
    };

    # Ensure cargo and rust are available for self-deployment
    environment.systemPackages = with pkgs; [
      rustc
      cargo
      pkg-config
      openssl
      git
    ];
  };
}
