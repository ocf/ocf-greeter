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
  };

  config = lib.mkIf cfg.enable {
    environment.etc."ocf-greeter/background.png".source = cfg.backgroundImage;

    services.greetd = {
      enable = true;
      settings.default_session.command = "${lib.getExe' pkgs.dbus "dbus-run-session"} ${lib.getExe pkgs.cage} -s -- ${lib.getExe cfg.package} ${config.services.displayManager.sessionData.desktops}/share/wayland-sessions ${cfg.defaultSession}";
    };
    security.pam.services.greetd.makeHomeDir = true;
  };
}
