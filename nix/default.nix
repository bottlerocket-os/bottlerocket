{ config ? import ./config.nix {}, nixpkgs ? import ./nixpkgs.nix }:
let
  tharNixpkgs = (nixpkgs { inherit config; }); # setup the nixpkgs closure for use with thar.
  nixpkgs' = tharNixpkgs {}; # configure nixpkgs for use (upstream config).

  overlaidPkgs = tharNixpkgs { overlays = [ (self: super: tharClosure self) ]; };

  tharClosure = self: let
    # thar closure's callPackage for resolving attributes from the
    # final set of attributes in `packages'.
    callPackage = (self.newScope packages);

    buildSupport = rec {
      # External dependencies can be pulled in using `pkgs'.
      pkgs = self;
      lib = pkgs.lib;
      overlaid = overlaidPkgs;

      # Explicit inclusion of common external dependency in scope, others
      # are provided directly as needed.
      inherit (pkgs)
        # The nixpkgs conventional derivation constructor -
        # specifically its non-gcc including alternative, which should
        # be folk's default.
        stdenvNoCC symlinkJoin writeScript fetchFromGitHub runCommand;


      # Provide config and the scope's callPackage for nested
      # derivations.
      inherit config callPackage;

      # TODO: this attribute should be based on the architecture being
      # targeted.
      inherit (config) base-container-image;

      # Docker CLI is the Nix pure docker derivation, but only the
      # docker cli to avoid having to copy down the entirety of the
      # docker closure (which requires containerd, dockerd, runc,
      # et. al). Bootstrapping or no-cache builds will still download
      # them, but those paths can be garbage collected.
      docker-cli = pkgs.callPackage ./docker/docker-cli.nix {};
      docker-run = callPackage ./docker/docker-run.nix {};
      docker-load = callPackage ./docker/docker-load.nix {};
      docker-image = callPackage ./docker/docker-image.nix {};
      docker-container = {
        setup = pkgs.callPackage ./docker/container-setup.nix {};
        teardown = pkgs.callPackage ./docker/container-teardown.nix {};
      };
      docker-sanity = callPackage ./docker/sanity-check.nix {};

      rpm-metadata = callPackage ./rpm/rpm-metadata.nix { inherit (pkgs) rpm; };
      rpm-macros = callPackage ./rpm/rpm-macros.nix { inherit (pkgs) rpm; };
      rpm-container = callPackage ./rpm/rpm-container.nix {};
      rpm-dependencies = callPackage ./rpm/rpm-dependency-resolver.nix {};
      rpmBuilder = callPackage ./rpm/rpm-builder.nix { inherit (pkgs) writeScript; };
      fetchRpmSources = callPackage ./rpm/fetch-rpm-sources.nix {};
      mkMacroPath = paths: builtins.concatStringsSep ":" paths;

      fetchCargo = callPackage ./rust/fetch-cargo.nix {};

      example = callPackage ./example/default.nix {};
    };
    buildPackages =  import ../packages { inherit callPackage; };
    packages = buildSupport // buildPackages;

  in packages;
in
tharClosure nixpkgs'
