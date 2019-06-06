let
# Load build config.
config = import ./config.nix {};
# Load platform details that may influence the build.
platform = import ./platform.nix { inherit config; };
# Download nixpkgs suitable for use, still needs "priming" in its own
# sense of configuration.
pinnedNixpkgs = import ./nixpkgs.nix { inherit config; };
in
rec {
  inherit config platform;

  nixpkgs = pinnedNixpkgs { config = {}; };
  lib = import ./lib { inherit nixpkgs; };

  container = import ./container { inherit nixpkgs; };
  playground = nixpkgs.callPackage ./playground.nix { inherit docker-cli; };
  docker-cli = nixpkgs.callPackage ./container/docker-cli.nix {};
  package-list = nixpkgs.callPackage ./thar/closure.nix {};
  rpm-metadata = nixpkgs.callPackage ./thar/rpm-metadata.nix { inherit platform nixpkgs; };
}
