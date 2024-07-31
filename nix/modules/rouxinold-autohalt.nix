{ lib
, pkgs
, config
, ...
}: with lib; {
  options.services.rouxinold-autohalt = {
    enable = mkEnableOption "rouxinold-autohalt";
    envFile = mkOption {
      type = types.str;
      default = "/opt/rouxinold/.env-autohalt";
    };
  };

  config = let
    rouxinold-autohalt = config.services.rouxinold-autohalt;
  in {
    systemd.services.rouxinold-autohalt = mkIf rouxinold-autohalt.enable {
      wants = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      enable = true;
      environment = {
        "ROUXINOLD_ENV_FILE" = rouxinold-autohalt.envFile;
      };
      serviceConfig = {
        ExecStart = "${pkgs.rouxinold-autohalt}/bin/rouxinold-autohalt";
        Restart = "always";
        User = "rouxinold-autohalt";
        Group = "rouxinold-autohalt";
      };
    };

    users.users.rouxinold-autohalt = {
      name = "rouxinold-autohalt";
      group = "rouxinold-autohalt";
      isNormalUser = true;
    };
    users.groups.rouxinold-autohalt = {};
  };
}
