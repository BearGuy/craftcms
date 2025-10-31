{
  description = "CraftCMS Rust service packaged via Nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, rust-overlay }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forEachSystem =
        nixpkgs.lib.genAttrs systems (
          system:
          let
            pkgs = import nixpkgs {
              inherit system;
              config.allowUnfree = true;
              overlays = [
                (import rust-overlay)
              ];
            };

            inherit (pkgs) lib;
            toolchain = pkgs.rust-bin.stable.latest.default;
            rustPlatform = pkgs.makeRustPlatform {
              cargo = toolchain;
              rustc = toolchain;
            };

            craftcms = rustPlatform.buildRustPackage {
              pname = "craftcms";
              version = "0.2.0";

              src = lib.cleanSourceWith {
                src = ./.;
                filter = lib.cleanSourceFilter;
              };

              cargoLock.lockFile = ./Cargo.lock;

              nativeBuildInputs = [
                pkgs.pkg-config
              ];

              buildInputs = lib.optionals pkgs.stdenv.isDarwin (
                with pkgs.darwin.apple_sdk.frameworks;
                [
                  Security
                  SystemConfiguration
                ]
              );

              meta = with lib; {
                description = "A simple CMS for managing and serving templated HTML documents";
                homepage = "https://github.com/bearguy/craftcms";
                license = licenses.mit;
                maintainers = [ ];
                mainProgram = "craftcms";
                platforms = systems;
              };
            };
          in
          {
            inherit system pkgs craftcms toolchain;
          }
        );
    in
    {
      packages = nixpkgs.lib.genAttrs systems (
        system:
        let
          inherit (forEachSystem.${system}) craftcms;
        in
        {
          default = craftcms;
          inherit craftcms;
        }
      );

      apps = nixpkgs.lib.genAttrs systems (
        system:
        {
          default = {
            type = "app";
            program = "${self.packages.${system}.craftcms}/bin/craftcms";
          };
        }
      );

      devShells = nixpkgs.lib.genAttrs systems (
        system:
        let
          inherit (forEachSystem.${system}) pkgs;
          inherit (pkgs) lib;
          inherit (forEachSystem.${system}) toolchain;
        in
        {
          default = pkgs.mkShell {
            packages =
              [
                toolchain
                pkgs.rust-bin.stable.latest.clippy
                pkgs.rust-bin.stable.latest.rustfmt
                pkgs.rust-bin.stable.latest.rust-src
                pkgs.rust-analyzer
                pkgs.pkg-config
              ]
              ++ lib.optionals pkgs.stdenv.isDarwin (
                with pkgs.darwin.apple_sdk.frameworks;
                [
                  Security
                  SystemConfiguration
                ]
              );

            RUST_SRC_PATH = "${pkgs.rust-bin.stable.latest.rust-src}/lib/rustlib/src/rust/library";
          };
        }
      );

      formatter = nixpkgs.lib.genAttrs systems (
        system:
        let
          inherit (forEachSystem.${system}) pkgs;
        in
        pkgs.nixfmt-rfc-style
      );

      checks = nixpkgs.lib.genAttrs systems (system: { default = self.packages.${system}.craftcms; });
    };
}
