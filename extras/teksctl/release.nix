{ pkgs ? import <nixpkgs> {} }:
let
  amis = {
    "0.1.3" = { amiID = "ami-0346bb6ef129f9f11"; };
    "0.1.4" = { amiID = "ami-0331423be16b32cca"; };
  };
in
rec {
  teksctlFor = pkgs.callPackage ./. {};
  archive = let
    imageRelease = "0.1.4";
    teksctl = teksctlFor { inherit (amis.${imageRelease}) amiID; versionExtra = "thar-${imageRelease}"; };
  in pkgs.runCommand "${teksctl.name}-thar-${imageRelease}-archive.tar.gz" {} ''
    tar -jcf $out -C ${teksctl}/bin eksctl teksctl
  '';
}
