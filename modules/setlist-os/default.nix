{ lib, impermanence, pkgs, ... }:

with lib;

let
  cfg = config.setlist-os;
in
{
  ###########################################################################
  options.setlist-os = {
    enable = mkEnableOption "Enable the Setlist-OS base profile";

    # ZFS layout knobs
    rootPool     = mkOption { type = types.str; default = "rpool"; };
    mediaPool    = mkOption { type = types.str; default = "mediapool"; };
    mediaDataset = mkOption { type = types.str; default = "library"; };
    mediaMount   = mkOption { type = types.str; default = "/media"; };
    persistMount = mkOption { type = types.str; default = "/persist"; };

    # dynamic-hostname feature
    hostname = {
      static       = mkOption { type = types.str; default = "setlist"; };
      useDynamic   = mkOption { type = types.bool; default = true; };
      dynamicFile  = mkOption { type = types.str; default = "/persist/hostname"; };
    };

    extraPersistentDirs = mkOption {
      type    = with types; listOf str;
      default = [ ];
    };
  };

  ###########################################################################
  config = mkIf cfg.enable {
    ################  Core Nix & pkgs  #######################################
    nix.settings.experimental-features = [ "nix-command" "flakes" ];
    nix.settings.auto-optimise-store   = true;
    nix.gc.automatic                   = true;

    environment.systemPackages = with pkgs; [
      rsync ripgrep tailscale git openssh
    ];

    ################  ZFS & boot  ###########################################
    boot.supportedFilesystems = [ "zfs" ];
    boot.zfs.devNodes         = "/dev/disk/by-partuuid";
    services.zfs.autoScrub.enable = true;

    boot.loader.systemd-boot.enable      = true;
    boot.loader.efi.canTouchEfiVariables = true;

    ################  Impermanence  #########################################
    imports = [ impermanence.nixosModules.impermanence ];

    environment.persistence."${cfg.persistMount}" = {
      directories =
        [ "/etc/ssh" "/var/lib/tailscale" "/var/log" "/home" ]
        ++ cfg.extraPersistentDirs;
      files = optional cfg.hostname.useDynamic cfg.hostname.dynamicFile;
    };

    ################  Media mount  ##########################################
    fileSystems."${cfg.mediaMount}" = {
      device = "${cfg.mediaPool}/${cfg.mediaDataset}";
      fsType = "zfs";
    };

    ################  Networking  ###########################################
    # compile-time placeholder; can be overridden at boot
    networking.hostName = cfg.hostname.static;

    # Runtime hostname setter
    systemd.services.setlist-dyn-hostname = mkIf cfg.hostname.useDynamic {
      description = "Set hostname from ${cfg.hostname.dynamicFile} at boot";
      wantedBy    = [ "multi-user.target" ];
      before      = [ "network.target" ];
      serviceConfig = {
        Type = "oneshot";
        ExecStart = ''
          ${pkgs.bash}/bin/bash -c '
            FILE="${cfg.hostname.dynamicFile}"
            if [ -f "$FILE" ]; then
              HN=$(cat "$FILE" | tr -d " \n\r")
              if [ -n "$HN" ]; then
                echo "setlist-dyn-hostname: setting hostname to $HN"
                /run/current-system/sw/bin/hostnamectl set-hostname "$HN"
              fi
            fi
          '
        '';
      };
    };

    ################  SSH & Tailscale  ######################################
    services.openssh.enable  = true;
    services.tailscale.enable = true;

    ################  Predefined service user  ##############################
    users.users.setlist = {
      isNormalUser   = true;
      extraGroups    = [ "wheel" ];
      hashedPassword = null;
      home           = "/home/setlist";
    };

    systemd.tmpfiles.rules = [
      "d ${cfg.mediaMount} 0775 setlist users - -"
    ];
  };
}

