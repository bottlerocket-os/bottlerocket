{ sources ? null, useFile ? false }:
let
  sources' = if useFile then (builtins.fromJSON (builtins.readFile sources)).sources else sources;
  fetchurl = import <nix/fetchurl.nix>;
in
map fetchurl sources'
