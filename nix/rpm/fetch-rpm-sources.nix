{ rpm-metadata }:
{ name, sources, spec }:
let
  metadata = rpm-metadata { inherit spec sources name; };
  sourcesJSON = "${metadata}/sources.json";
  sourcesList = (builtins.fromJSON (builtins.readFile sourcesJSON)).sources;
  fetchurl = import <nix/fetchurl.nix>;
in
map fetchurl sourcesList
