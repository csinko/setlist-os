#!/usr/bin/env bash
#-------------------------------------------------------------------------------
#  Setlist-OS Bootstrap Wizard
#  • Partitions two NVMe drives
#  • Creates ZFS pools/datasets
#  • Clones the setlist-os repo and installs NixOS
#  • On error, tears down any ZFS pools and unmounts /mnt
#-------------------------------------------------------------------------------
set -euo pipefail

# ── Enable non-free packages for this session ────────────────────────────────
export NIXPKGS_ALLOW_UNFREE=1

# ── Color palette ─────────────────────────────────────────────────────────────
C_RESET="\033[0m"
C_INFO="\033[0;32m"
C_WARN="\033[0;33m"
C_ERR="\033[0;31m"
C_CMD="\033[0;34m"

log()     { printf "${C_INFO}[INFO] %s${C_RESET}\n" "$*"; }
warn()    { printf "${C_WARN}[WARN] %s${C_RESET}\n" "$*"; }
error()   { printf "${C_ERR}[ERR ] %s${C_RESET}\n" "$*"; }
run_cmd() { printf "${C_CMD}[CMD ] %s${C_RESET}\n" "$*"; eval "$*"; }

# ── Log everything to file too ────────────────────────────────────────────────
LOGFILE="/tmp/setlist-bootstrap-$(date +%F-%H%M%S).log"
exec > >(tee -a "$LOGFILE") 2>&1

# ── Cleanup function ──────────────────────────────────────────────────────────
cleanup() {
  log "Cleaning up mounts and ZFS pools..."
  run_cmd "umount -R /mnt || true"
  run_cmd "zpool destroy -f mediapool || true"
  run_cmd "zpool destroy -f rpool    || true"
}

# ── Error handler ─────────────────────────────────────────────────────────────
on_error() {
  error "Bootstrap failed at line $1. Performing cleanup..."
  cleanup
  error "See log at $LOGFILE"
  exit 1
}
trap 'on_error $LINENO' ERR

# ── Gather user input ─────────────────────────────────────────────────────────
log "Block devices currently detected:"
run_cmd lsblk -o NAME,SIZE,FSTYPE,LABEL,MODEL,TYPE,MOUNTPOINT

read -rp "OS NVMe device  [default: /dev/nvme0n1]: " OSDEV
OSDEV=${OSDEV:-/dev/nvme0n1}
read -rp "Media NVMe device [default: /dev/nvme1n1]: " MEDIADEV
MEDIADEV=${MEDIADEV:-/dev/nvme1n1}

[[ -b "$OSDEV" ]]    || { error "$OSDEV not a block device"; exit 1; }
[[ -b "$MEDIADEV" ]] || { error "$MEDIADEV not a block device"; exit 1; }

log "Planned actions:"
echo "  • PARTITION  $OSDEV  (EFI + ZFS root)"
echo "  • PARTITION  $MEDIADEV (ZFS media)"
echo "  • CREATE     rpool      on $OSDEV"
echo "  • CREATE     mediapool  on $MEDIADEV"
echo "  • Mount, clone repo, install NixOS"

warn "ALL DATA on these drives will be destroyed."
read -rp "Type YES to continue: " CONFIRM
[[ $CONFIRM == YES ]] || { error "Aborted by user."; exit 1; }

# ── Partition disks ──────────────────────────────────────────────────────────
log "Partitioning $OSDEV ..."
run_cmd parted -s "$OSDEV" mklabel gpt
run_cmd parted -s "$OSDEV" mkpart ESP fat32 1MiB 513MiB
run_cmd parted -s "$OSDEV" set 1 esp on
run_cmd parted -s "$OSDEV" mkpart primary 513MiB 100%
run_cmd parted -s "$OSDEV" print

log "Partitioning $MEDIADEV ..."
run_cmd parted -s "$MEDIADEV" mklabel gpt
run_cmd parted -s "$MEDIADEV" mkpart primary 1MiB 100%
run_cmd parted -s "$MEDIADEV" print

read -rp "Partitions look OK? [y/N] " ok
[[ ${ok:-n} =~ ^[Yy]$ ]] || { error "User aborted."; exit 1; }

# ── Create ZFS pools ─────────────────────────────────────────────────────────
run_cmd modprobe zfs

log "Creating rpool ..."
run_cmd zpool create -f -o ashift=12 \
       -O compression=zstd -O atime=off -O mountpoint=none \
       rpool "${OSDEV}p2"
for ds in root nix persist; do
  run_cmd zfs create -o mountpoint=legacy rpool/$ds
done

log "Creating mediapool ..."
run_cmd zpool create -f -o ashift=12 \
       -O compression=zstd -O atime=off \
       mediapool "${MEDIADEV}p1"
run_cmd zfs create -o mountpoint=legacy mediapool/library

# ── Mount hierarchy ──────────────────────────────────────────────────────────
log "Mounting filesystems ..."
run_cmd mount -t zfs rpool/root /mnt
for d in nix persist media boot; do mkdir -p /mnt/$d; done
run_cmd mount -t zfs rpool/nix         /mnt/nix
run_cmd mount -t zfs rpool/persist     /mnt/persist
run_cmd mount -t zfs mediapool/library /mnt/media
run_cmd mount "${OSDEV}p1" /mnt/boot

# ── Clone repo & commit hardware config ─────────────────────────────────────
log "Cloning setlist-os ..."
run_cmd git clone https://github.com/csinko/setlist-os /mnt/etc/nixos

run_cmd nixos-generate-config --root /mnt --no-filesystems
run_cmd git -C /mnt/etc/nixos add hardware-configuration.nix
run_cmd 'git -C /mnt/etc/nixos -c user.name="setlist-bootstrap" \
       -c user.email="bootstrap@setlist-os.local" \
       commit -m "Add hardware configuration" --no-gpg-sign'

read -rp "Hostname for this box: " HN
run_cmd "echo $HN > /mnt/persist/hostname"

# ── Install NixOS ────────────────────────────────────────────────────────────
log "Installing NixOS (this builds the ZFS module; may take a while) ..."
run_cmd nixos-install --root /mnt --flake /mnt/etc/nixos#setlist-os

log "SUCCESS!  Installation complete."
log "Next steps:"
echo "  sudo reboot"

