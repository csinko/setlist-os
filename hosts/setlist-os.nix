{ pkgs, ... }: {
  imports = [ 
    ../hardware-configuration.nix
  ];

  setlist-os.enable = true;

  # Disk mounts (matches earlier partitioning)
  fileSystems."/"        = { device = "rpool/root";    fsType = "zfs"; };
  fileSystems."/nix"     = { device = "rpool/nix";     fsType = "zfs"; };
  fileSystems."/persist" = { device = "rpool/persist"; fsType = "zfs"; neededForBoot = true; };
  fileSystems."/boot"    = {
    device  = "/dev/disk/by-partlabel/ESP";
    fsType  = "vfat";
    options = [ "nofail" "umask=0077" ];
  };

  # Safe GPU boot first time
  # boot.kernelPackages = pkgs.linuxPackages_latest;
  boot.kernelParams   = [ "quiet" "nomodeset" ];

  networking.hostId = "00000000";   # placeholder (keeps derivations identical)

  boot.initrd.preDeviceCommands = ''
    # Generate an 8-hex hostId from the pool GUID. Any deterministic rule works.
    if [ "$hostId" = "00000000" ]; then
        if hostId=$(zpool list -H -o guid rpool 2>/dev/null | cut -c1-8); then
            printf '%08s' "$hostId" | dd of=/etc/hostid bs=1 count=8 conv=ucase 2>/dev/null
            printf "[initrd] regenerated hostid to 0x%s\n" "$hostId"
        fi
    fi
  '';
}

