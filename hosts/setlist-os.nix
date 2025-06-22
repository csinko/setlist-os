{ pkgs, ... }: {
  imports = [ 
    ../modules/setlist-os
    ./hardware-configuration.nix
  ];

  setlist-os.enable = true;

  # Disk mounts (matches earlier partitioning)
  fileSystems."/"        = { device = "rpool/root";    fsType = "zfs"; };
  fileSystems."/nix"     = { device = "rpool/nix";     fsType = "zfs"; };
  fileSystems."/persist" = { device = "rpool/persist"; fsType = "zfs"; };
  fileSystems."/home"    = { device = "rpool/home";    fsType = "zfs"; };
  fileSystems."/boot"    = {
    device  = "/dev/disk/by-partlabel/ESP";
    fsType  = "vfat";
    options = [ "nofail" "umask=0077" ];
  };

  # Safe GPU boot first time
  boot.kernelPackages = pkgs.linuxPackages_latest;
  boot.kernelParams   = [ "quiet" "nomodeset" ];
}

