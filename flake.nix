{
  description = "ocf-greeter";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-26.05/nixexprs.tar.zst";
    nixpkgs-unstable.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.zst";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      nixpkgs-unstable,
      crane,
      flake-utils,
      treefmt-nix,
    }:
    let
      overlay = final: prev: {
        ocf-greeter = final.callPackage ./. {
          craneLib = crane.mkLib final;
        };
      };
    in
    {
      overlays.default = overlay;
      nixosModules.default = { ... }: {
        imports = [ ./module.nix ];
        nixpkgs.overlays = [ overlay ];
      };

      nixosConfigurations.test = nixpkgs-unstable.lib.nixosSystem {
        system = "x86_64-linux";
        modules = nixpkgs-unstable.lib.singleton (
          {
            pkgs,
            lib,
            config,
            ...
          }:
          {
            nixpkgs.overlays = [ overlay ];
            services.greetd = {
              enable = true;
              settings = {
                terminal.vt = 1;
                default_session = {
                  command = "${lib.getExe pkgs.cage} -s -- ${lib.getExe pkgs.ocf-greeter} ${config.services.displayManager.sessionData.desktops}/share/wayland-sessions Sway";
                  user = "greeter";
                };
              };
            };

            programs.sway.enable = true;

            users = {
              groups.test = { };
              groups.greeter = { };
              groups.users = { };
              users.test = {
                isNormalUser = true;
                password = "test";
                createHome = true;
                group = "users";
                extraGroups = [ "wheel" ];
              };
              users.greeter = {
                isSystemUser = true;
                group = "greeter";
              };
            };
            security.sudo.wheelNeedsPassword = false;

            system.stateVersion = "26.05";
          }
        );
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        # see https://github.com/iced-rs/iced/blob/master/DEPENDENCIES.md
        dlopenLibraries = with pkgs; [
          libxkbcommon
          vulkan-loader
          wayland
        ];

        pkgs = import nixpkgs {
          inherit system;
          overlays = [ overlay ];
        };

        treefmtConfig = treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs.nixfmt.enable = true;
          programs.rustfmt.enable = true;
        };
      in
      {
        packages.default = pkgs.ocf-greeter;

        formatter = treefmtConfig.config.build.wrapper;
        checks.formatting = treefmtConfig.config.build.check self;

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
          ];
          env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath dlopenLibraries}";
          nativeBuildInputs = [ treefmtConfig.config.build.wrapper ];
        };
      }
    );
}
