{ pkgs ? import <nixpkgs> {} }:
rec {
  teksctl = pkgs.callPackage ./. {};
  archive = pkgs.runCommand "${teksctl.name}-archive.tar.gz" {} ''
    tar -jcf $out -C ${teksctl}/bin eksctl teksctl
  '';
}
