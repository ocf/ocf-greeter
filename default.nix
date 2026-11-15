{
  pkgs,
  lib,
  craneLib,
  symlinkJoin,
}:

let
  pname = "ocf-greeter";
  version = "0.0.1";
  meta = with lib; {
    description = "Greeter for OCF desktops";
    homepage = "https://github.com/ocf/ocf-greeter";
    platforms = platforms.linux;
  };
  unwrapped = craneLib.buildPackage {
    inherit pname version meta;
    src = craneLib.cleanCargoSource ./.;
  };
in
symlinkJoin {
  inherit pname version meta;
  paths = [ unwrapped ];

  buildInputs = with pkgs; [
    makeWrapper
  ];

  postBuild = ''
    wrapProgram $out/bin/ocf-greeter \
      --prefix LD_LIBRARY_PATH : ${
        lib.makeLibraryPath (
          with pkgs;
          [
            libxkbcommon
            vulkan-loader
            wayland
          ]
        )
      }
  '';
}
