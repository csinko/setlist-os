{
  description = "Setlist-OS – music-server base";

  inputs = {
    nixpkgs      .url = "github:nixos/nixpkgs/nixos-unstable";
    impermanence .url = "github:nix-community/impermanence";
    rust-overlay .url = "github:oxalica/rust-overlay";
    flake-utils  .url = "github:numtide/flake-utils";
  };

  outputs = inputs@{ self, nixpkgs, impermanence, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        dockerPkgs =
          if pkgs.stdenv.hostPlatform.isLinux
          then [ pkgs.docker pkgs.docker-compose ]
          else [ ];
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rustfmt" "clippy" ];
            })
            chromaprint
            ffmpeg
            sqlx-cli
            postgresql_16
            rust-analyzer
            pkg-config
            openssl
          ] ++ dockerPkgs ++ (with pkgs.nodePackages; [ pnpm concurrently ]);

          shellHook = ''
            export DATABASE_URL="postgres://setlist:setlist@localhost/setlist"
            export AMQP_URL="amqp://setlist:setlist@localhost:5672/%2f"
            export SQLX_OFFLINE=1
            echo "Dev shell ready for ${system}"
          '';
        };
      })
    //
    {
      nixosModules.setlist-os = ./modules/setlist-os;
      nixosConfigurations =
        let
          linuxSystems = [ "x86_64-linux" "aarch64-linux" ];
        in
          builtins.listToAttrs (map
            (system: {
              name = "setlist-os-${system}";
              value = nixpkgs.lib.nixosSystem {
                inherit system;
                specialArgs = { inherit impermanence; };
                modules = [
                  self.nixosModules.setlist-os
                  ./hosts/setlist-os.nix
                ];
              };
            })
          linuxSystems);
    };
}

