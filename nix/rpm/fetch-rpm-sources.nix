{ rpm-metadata }:
{ sources, spec }:
let
  metadata = rpm-metadata { inherit spec sources; };
  sourcesJSON = "${metadata}/sources.json";
  sourcesList = (builtins.fromJSON (builtins.readFile sourcesJSON)).sources;
  fetchurl = import <nix/fetchurl.nix>;
in
map fetchurl sourcesList
