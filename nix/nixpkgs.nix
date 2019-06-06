{ config ? import ./config.nix {}, ... }:
let
  cfg = config.deps.nixpkgs;
in
assert builtins.isString cfg.sha256;
assert builtins.isString cfg.commit;

let nixpkgs = builtins.fetchTarball {
  inherit (cfg) sha256;
  url = "https://github.com/NixOS/nixpkgs/archive/${cfg.commit}.tar.gz";
};
in
# Expose pinned nixpkgs for builds.
import nixpkgs

