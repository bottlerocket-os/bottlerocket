{ lib, rpm-metadata }:
{ name, sources, spec }:
let
  remoteSources = let
    metadata = rpm-metadata { inherit name sources spec; };
  in
    metadata.sources;
  fetchurl = import <nix/fetchurl.nix>;
in

with lib;
assert assertMsg (isList remoteSources)
  "Resolved sources from ${name} were not a list";
let
  sourceEntries = flatten (map attrValues remoteSources);
  # Check that its not null, and if so, that it's not an empty string.
  check = v: v != null -> v != "";
  noEmptyEntry = all check sourceEntries;
in
assert assertMsg noEmptyEntry
  "Resolved sources have empty data parsed";

map fetchurl remoteSources
