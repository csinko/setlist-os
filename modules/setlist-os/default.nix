{ options, config, lib, pkgs, impermanence, ... }:

with lib;

let
  cfg = config.setlist-os;
in
{
  # ──────────────────────────────────────────────────────────────────────────
  # Imports
  # ──────────────────────────────────────────────────────────────────────────
  imports = [
    impermanence.nixosModules.impermanence
  ];

  # ──────────────────────────────────────────────────────────────────────────
  # Module options
  # ──────────────────────────────────────────────────────────────────────────
  options.setlist-os = {
    enable = mkEnableOption "Enable the Setlist-OS base profile";

    rootPool     = mkOption { type = types.str; default = "rpool"; };
    mediaPool    = mkOption { type = types.str; default = "mediapool"; };
    mediaDataset = mkOption { type = types.str; default = "library"; };
    mediaMount   = mkOption { type = types.str; default = "/media"; };
    persistMount = mkOption { type = types.str; default = "/persist"; };

    hostname = {
      static      = mkOption { type = types.str; default = "setlist"; };
      useDynamic  = mkOption { type = types.bool; default = true; };
      dynamicFile = mkOption { type = types.str; default = "/etc/setlist-hostname"; };
    };

    extraPersistentDirs = mkOption {
      type    = with types; listOf str;
      default = [ ];
    };
  };

  # ──────────────────────────────────────────────────────────────────────────
  # Implementation
  # ──────────────────────────────────────────────────────────────────────────
  config = mkIf cfg.enable {
    # Core Nix settings
    nix.settings.experimental-features = [ "nix-command" "flakes" ];
    nix.settings.auto-optimise-store   = true;
    nix.gc.automatic                   = true;

    # Allow non-free packages (e.g. nixFlakes, tailscale)
    nixpkgs.config.allowUnfree         = true;

    environment.systemPackages = with pkgs; [
      rsync ripgrep tailscale git openssh
    ];

    # ZFS & boot loader
    boot.supportedFilesystems        = [ "zfs" ];
    boot.zfs.devNodes                = "/dev/disk/by-partuuid";
    services.zfs.autoScrub.enable    = true;
    boot.loader.systemd-boot.enable  = true;
    boot.loader.efi.canTouchEfiVariables = true;

    boot.zfs.forceImportAll = true;
    boot.zfs.extraPools = [ cfg.mediaPool ];

    # Impermanence: only /persist survives
    environment.persistence."${cfg.persistMount}" = {
      directories = [
        "/var/lib/tailscale"
        "/var/log"
        "/home"
        "/var/lib/setlist"
        "/var/lib/nixos"
      ] ++ cfg.extraPersistentDirs;
      files = optional cfg.hostname.useDynamic cfg.hostname.dynamicFile ++ [
        "/etc/ssh/ssh_host_ed25519_key"
        "/etc/ssh/ssh_host_ed25519_key.pub"
        "/etc/ssh/ssh_host_rsa_key"
        "/etc/ssh/ssh_host_rsa_key.pub"
      ];
    };

    # Media dataset auto-mount
    fileSystems."${cfg.mediaMount}" = {
      device = "${cfg.mediaPool}/${cfg.mediaDataset}";
      fsType = "zfs";
    };

    # Hostname (static placeholder)
    networking.hostName = cfg.hostname.static;

    environment.etc."setlist-dyn-hostname.sh" = {
      text = ''
        #!${pkgs.bash}/bin/bash
        file='${cfg.hostname.dynamicFile}'
        if [[ -r "$file" ]]; then
          hn=$(tr -d " \r\n" < "$file")
          if [[ -n $hn ]]; then
            /run/current-system/sw/bin/hostname "$hn"
          fi
        fi
      '';
      mode = "0755";
    };

    systemd.services.setlist-dyn-hostname = mkIf cfg.hostname.useDynamic {
      description = "Set runtime hostname from ${cfg.hostname.dynamicFile}";
      wantedBy    = [ "sysinit.target" ];   # runs early, before network
      before      = [ "network-pre.target" ];
      serviceConfig = {
        Type      = "oneshot";
        ExecStart = [ "/etc/setlist-dyn-hostname.sh" ];
      };
    };


    # Networking services
    services.openssh.enable  = true;
    services.tailscale.enable = true;

    # Predefined service user
    users.users.setlist = {
      isSystemUser  = true;                  # ← marks it as a service account
      group         = "setlist";
      description   = "Setlist-OS media service user";

      home          = "/var/lib/setlist";    # service data lives here
      createHome    = true;

      shell         = "${pkgs.shadow}/bin/nologin";
      hashedPassword = "*";                  # “*” = no password accepted
    };

    users.groups.setlist = { };

    services.logrotate.enable = true;


    # Ensure /media is writable by setlist
    systemd.tmpfiles.rules = [
      "d ${cfg.mediaMount} 0775 setlist users - -"
    ];
  };
}

