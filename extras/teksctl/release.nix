{ pkgs ? import <nixpkgs> {} }:
{
  teksctl = pkgs.callPackage ./. {};
}
