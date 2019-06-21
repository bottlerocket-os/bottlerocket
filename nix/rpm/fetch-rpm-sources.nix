{ rpm-metadata }:
{ name, sources, spec }:
let
  packageMetadata = rpm-metadata { inherit spec sources name; };
  fetchurl = import <nix/fetchurl.nix>;
in
map fetchurl packageMetadata.sources
