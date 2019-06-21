# You'll need to provide a few things to run - add an SSH key below -
# then you can setup a builder!
#
# KEY_NAME=
# INSTANCE_TYPE=
# INSTANCE_PROFILE=
# SECURITY_GROUP=
#
# aws ec2 --region us-west-2 run-instances --image-id ami-03f8a737546e47fb0 \
# 	--key-name $KEY_NAME  --instance-type $INSTANCE_TYPE \
# 	--iam-instance-profile Name=$INSTANCE_PROFILE \
# 	--security-groups $SECURITY_GROUP \
# 	--tag-specification 'ResourceType=instance,Tags=[{Key=Name,Value=tharnix}]' \
# 	--block-device-mapping 'DeviceName=/dev/sda1,Ebs={VolumeSize=100,VolumeType=gp2}' \
# 	--user-data file://nix/user-data.nix
#
{ config, lib, pkgs, ... }:
with lib;
let
  region = "us-west-2";
in
{
  imports = [
    <nixpkgs/nixos/modules/virtualisation/amazon-image.nix>
  ];

  # Force the user-data configuration loader to stop.
  systemd.services.amazon-init.enable = mkForce false;

  services.sysstat.enable = true;
  services.telegraf.enable = true;
  services.telegraf.extraConfig = {
    inputs = {
      disk = {
        ignore_fs = ["devfs" "devtmpfs"];
      };
      diskio = { };
      processes = { };
      docker = { };
      sysstat = {
        sadc_path = "${pkgs.sysstat}/lib/sa/sadc";
        sadf_path = "${pkgs.sysstat}/bin/sadf";
        activities = [ "DISK" ];
        group = true;
      };
    };

    outputs = {
      # Write metrics out to cloudwatch.
      cloudwatch = {
        inherit region;
        namespace = "tharnix";
      };
    };
  };

  networking.firewall = {
    enable = true;
    allowPing = true;
  };

  services.openssh = {
    enable = true;
    challengeResponseAuthentication = false;
    passwordAuthentication = false;
  };
  security.sudo.wheelNeedsPassword = false;

  virtualisation.docker.enable = true;

  environment.systemPackages = with pkgs; [
    htop vim emacs26-nox strace
  ];

  # Automatically keep the system up to date.
  system.autoUpgrade.enable = true;
  system.nixos.label = "tharnix";

  nix = {
    gc.automatic = true;
    gc.options = "--delete-older-than 30d";
    optimise.automatic = true;

    maxJobs = 8;
    extraOptions = ''
    # maximum time a build can go without producing *anything* on stdout/stderr
    max-silent-time = 300
    # Keep outputs for indirectly referenced derivations in gcroots (garbage collection)
    keep-outputs = true
    # Keep related derivations for indirectly referenced derivations in gcroots (garbage collection)
    keep-derivations = true
    '';

    trustedUsers = [ "@wheel" "builder" "@builder" ];
    sandboxPaths = [ "/run/docker.sock" "/var/run/docker.sock" ];
  };

  # Assign the docker group also to the build users for them to make builds happen inside of containers.
  users.groups.docker.members = let
    nixbldUsers = lib.filterAttrs (n: v: v.group == "nixbld") config.users.users;
  in
    builtins.attrNames nixbldUsers;

  users.users.builder = {
    isNormalUser = true;
    extraGroups = [ "builder" "docker" "wheel" ];
    openssh.authorizedKeys.keys = [
       # insert authorized key here
    ];
  };

  users.users.telegraf.extraGroups = [ "docker" ];
}
