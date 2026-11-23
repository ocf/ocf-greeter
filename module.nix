{
  config,
  inputs,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.ocf-greeter;
in
{
  options.services.ocf-greeter = {
    enable = lib.mkEnableOption "ocf greeter";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.ocf-greeter;
      defaultText = lib.literalExpression "pkgs.ocf-greeter";
      description = "The ocf-greeter package to use.";
    };

    backgroundImage = lib.mkOption {
      type = lib.types.path;
      default = ./assets/background.png;
      description = "background image for the login screen";
    };

    defaultSession = lib.mkOption {
      type = lib.types.str;
      description = "the default session to select in the dropdown";
    };

    displayScale = lib.mkOption {
      type = lib.types.float;
      default = 1.0;
      description = "the display scale to pass to cage";
    };
  };

  # use unstable greetd module to have correct pam settings (inheriting `login` modules)
  # FIXME: remove after 26.11 upgrade
  disabledModules = [
    "services/display-managers/greetd.nix"
  ];
  imports = [
    "${inputs.nixpkgs-unstable}/nixos/modules/services/display-managers/greetd.nix"
  ];

  config = lib.mkIf cfg.enable (
    let
      kanshi-conf = pkgs.writeText "kanshi-conf" ''
        profile {
          output * enable scale ${lib.strings.floatToString cfg.displayScale}
        }
      '';
      display-scale = (
        pkgs.writeShellScript "display-scale" ''
          ${lib.getExe pkgs.kanshi} --config ${kanshi-conf} &
          exec "$@"
        ''
      );
      greeterCommand = "${lib.getExe' pkgs.dbus "dbus-run-session"} ${lib.getExe pkgs.cage} -s -- ${display-scale} ${lib.getExe cfg.package} ${config.services.displayManager.sessionData.desktops}/share/wayland-sessions ${cfg.defaultSession}";

      greetdConfig = (pkgs.formats.toml { }).generate "greetd.toml" {
        terminal.switch = false;
        default_session = {
          command = greeterCommand;
          user = "greeter";
        };
      };

      mkGreetd =
        vtNum:
        lib.nameValuePair "autovt@tty${toString vtNum}" {
          description = "greetd on tty${toString vtNum}";
          after = [
            "systemd-user-sessions.service"
            "plymouth-quit-wait.service"
          ];
          conflicts = [ "getty@tty${toString vtNum}.service" ];
          serviceConfig = {
            ExecStart = "${pkgs.greetd}/bin/greetd --config ${greetdConfig} --vt ${toString vtNum}";
            Restart = "no";

            StandardInput = "tty";
            StandardOutput = "journal";
            StandardError = "journal";
            TTYPath = "/dev/tty${toString vtNum}";
            TTYReset = true;
            TTYVHangup = true;
          };
        };
    in
    {
      environment.etc."ocf-greeter/background.png".source = cfg.backgroundImage;

      # greetd for tty1
      services.greetd = {
        enable = true;
        settings.default_session.command = greeterCommand;
      };

      # fill tty 2-5 on demand with greetd (reserve tty6 for getty)
      systemd.services = builtins.listToAttrs (map mkGreetd (lib.range 2 5));
    }
  );
}
