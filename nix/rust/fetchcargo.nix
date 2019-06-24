{ pkgs }:
let
  fetchcargo = pkgs.callPackage "${pkgs.path}/pkgs/build-support/rust/fetchcargo.nix" {};
in
{ name, src, srcs ? [], sourceRoot ? null, patches ? [], cargoUpdateHook ? "", sha256 }:
fetchcargo { inherit name src srcs patches sourceRoot cargoUpdateHook sha256; }

