{
  config,
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

  disabledModules = [
    "services/display-managers/greetd.nix"
  ];
  imports = [
    ./greetd.nix
  ];

  config = lib.mkIf cfg.enable {
    environment.etc."ocf-greeter/background.png".source = cfg.backgroundImage;

    services.greetd =
      let
        display-scale = (
          pkgs.writeShellScript "display-scale" ''
            for out in $(${lib.getExe pkgs.wlr-randr} --json | ${lib.getExe pkgs.jq} -r '.[].name'); do
              ${lib.getExe pkgs.wlr-randr} --output "$out" --scale ${lib.strings.floatToString cfg.displayScale}
            done

            exec "$@"
          ''
        );
      in
      {
        enable = true;
        settings.default_session.command = "${lib.getExe' pkgs.dbus "dbus-run-session"} ${lib.getExe pkgs.cage} -s -- ${display-scale} ${lib.getExe cfg.package} ${config.services.displayManager.sessionData.desktops}/share/wayland-sessions ${cfg.defaultSession}";
      };
  };
}
