{
  description = "Setlist-OS • ZFS + impermanence music-server base";

  inputs = {
    nixpkgs      .url = "github:nixos/nixpkgs/nixos-unstable";
    impermanence .url = "github:nix-community/impermanence";
  };

  outputs = { self, nixpkgs, impermanence, ... }: {
    nixosModules.setlist-os = ./modules/setlist-os;

    nixosConfigurations.setlist-os = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      specialArgs = { inherit impermanence; };
      modules = [
        self.nixosModules.setlist-os
        ./hosts/setlist-os.nix
      ];
    };
  };
}

