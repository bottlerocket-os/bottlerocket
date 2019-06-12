let
# Load build config.
config = import ./config.nix {};

# Use a pinned copy of nixpkgs for build.
nixpkgs = (import ./nixpkgs.nix { inherit config; })
            { config = {}; overlays = [ ]; };
thar = self: let
  callPackage = self.newScope packages;
  packages = rec {
    inherit config callPackage;
  
    # External dependencies can be pulled in using `pkgs'.
    pkgs = self;
    lib = pkgs.lib;
    
    # Explicit inclusion of common external dependency in scope, others
    # are provided directly as needed.
    inherit (pkgs)
      # The nixpkgs conventional derivation constructor.
      stdenv;
    
    docker-cli = callPackage ./docker/docker-cli.nix { inherit (pkgs) docker; };
    docker-run = callPackage ./docker/docker-run.nix {};
    docker-load = callPackage ./docker/docker-load.nix {};
    docker-image = callPackage ./docker/docker-image.nix {};
      
    rpm-metadata = pkgs.callPackage ./rpm/rpm-metadata.nix { inherit (pkgs) rpm; };
    rpm-macros = pkgs.callPackage ./rpm/rpm-macros.nix { inherit (pkgs) rpm; };
    rpm-builder = pkgs.callPackage ./rpm/rpm-builder.nix {};
    fetchRpmSources = import ./rpm/fetch-rpm-sources.nix;
    mkMacroPath = paths: builtins.concatStringsSep ":" paths;
  
    example = callPackage ./example/default.nix {};
    
  };
  in packages;

in
thar nixpkgs
