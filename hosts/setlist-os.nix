{ pkgs, ... }: {
  imports = [
    ../hardware-configuration.nix
  ];

  ##############################################################################
  # Enable the appliance
  ##############################################################################
  setlist-os.enable = true;

  ##############################################################################
  # Filesystems (matches bootstrap layout)
  ##############################################################################
  fileSystems."/"        = { device = "rpool/root";    fsType = "zfs"; };
  fileSystems."/nix"     = { device = "rpool/nix";     fsType = "zfs"; };
  fileSystems."/persist" = {
    device        = "rpool/persist";
    fsType        = "zfs";
    neededForBoot = true;
  };
  fileSystems."/boot"    = {
    device  = "/dev/disk/by-partlabel/ESP";
    fsType  = "vfat";
    options = [ "nofail" "umask=0077" ];
  };

  ##############################################################################
  # Kernel / initrd tweaks
  ##############################################################################
  boot.kernelParams = [ "quiet" "nomodeset" ];

  networking.hostId = "00000000";   # placeholder (regenerated in initrd)

  boot.initrd.preDeviceCommands = ''
    # Generate an 8-hex hostId from the pool GUID (deterministic per box).
    if [ "$hostId" = "00000000" ]; then
      if hostId=$(zpool list -H -o guid rpool 2>/dev/null | cut -c1-8); then
        printf '%08s' "$hostId" | dd of=/etc/hostid bs=1 count=8 conv=ucase 2>/dev/null
        printf "[initrd] regenerated hostid to 0x%s\n" "$hostId"
      fi
    fi
  '';

  ##############################################################################
  # Users
  ##############################################################################
  users.users.csinko = {
    isNormalUser   = true;
    extraGroups    = [ "wheel" ];        # sudo-capable
    hashedPassword = "$6$dzNl14C5CYlCJd.l$eAgJ.BFbRRBkOZsiqOiP7ZnXhhc.qsHGf3ozUezY/WqE0hpIog.FsXX3esm8vOGp4vSPwAiRz6OraraOJEkMT1";
    shell          = pkgs.zsh;           # or pkgs.bashInteractive
  };

  # service account comes from the appliance module (setlist user)

  ##############################################################################
  # OpenSSH – allow password auth for csinko
  ##############################################################################
  services.openssh = {
    enable = true;
    settings = {
      PasswordAuthentication = "yes";
      PermitRootLogin        = "no";
    };
  };
}

