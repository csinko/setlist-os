{
  description = "Setlist-OS • ZFS + impermanence music-server base";

  inputs = {
    nixpkgs      .url = "github:nixos/nixpkgs/nixos-unstable";
    impermanence .url = "github:nix-community/impermanence";
    flake-utils  .url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, impermanence, flake-utils, ... }:
    let
      lib      = nixpkgs.lib;
      pkgs = import nixpkgs { system = "x86_64-linux"; };
    in
  {
    ############################################################################
    # 1) exported module library
    ############################################################################
    nixosModules.setlist-os = import ./modules/setlist-os {
      inherit lib impermanence pkgs;
    };

    ############################################################################
    # 4) DEMO host (yours) – optional, but included for CI & ISO build
    ############################################################################
    nixosConfigurations.setlist-os = lib.nixosSystem {
      system  = "x86_64-linux";
      modules = [
        self.nixosModules.setlist-os     # core appliance
        ./hosts/setlist-os.nix            # your host specifics
      ];
    };
  };
}

