{ pkgs ? import <nixpkgs> {} }:
let
  imageRelease = "0.1.6";
in
rec {
  teksctl = pkgs.callPackage ./. { versionExtra = "thar-${imageRelease}"; };
  archive = pkgs.runCommand "${teksctl.name}-thar-${imageRelease}-archive.tar.gz" {} ''
    tar -jcf $out -C ${teksctl}/bin eksctl teksctl
  '';
}
