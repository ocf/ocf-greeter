{
  description = "ocf-greeter";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-26.05/nixexprs.tar.zst";
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
