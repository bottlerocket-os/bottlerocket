{ config ? import ./config.nix {}, nixpkgs ? import ./nixpkgs.nix }:
let
  tharNixpkgs = (nixpkgs { inherit config; }); # setup the nixpkgs closure for use with thar.
  nixpkgs' = tharNixpkgs {}; # configure nixpkgs for use (upstream config).

  overlaidPkgs = tharNixpkgs { overlays = [ (self: super: tharClosure self) ]; };

  # self is the "foundational" scope for this to consume.
  tharClosure = self: let
    # thar closure's callPackage for resolving attributes from the
    # final set of attributes in `packages'.
    callPackage = (self.newScope closure);
    # thar *buildSupport* closure which limits its scope for injected
    # attrs to the defined buildSupport attrset.
    callBuildPackage = (self.newScope buildSupport);

    # Supporting derivation and tooling.
    buildSupport = rec {
      # External dependencies can be pulled in using `pkgs'.
      pkgs = self;
      # Helpful additions on top of Nix's builtins, quality of life
      # addition.
      lib = pkgs.lib;
      # nixpkgs with a thar overlay.
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

      # Include tharPackages for injecting into buildSupport scoped
      # derivations.
      inherit (buildPackages) tharPackages;

      # Docker CLI is the Nix pure docker derivation, but only the
      # docker cli to avoid having to copy down the entirety of the
      # docker closure (which requires containerd, dockerd, runc,
      # et. al). Bootstrapping or no-cache builds will still download
      # them, but those paths can be garbage collected.
      docker-cli = pkgs.callPackage ./docker/docker-cli.nix {};
      # Load docker image, may be dropped in favor of passthru helpers
      # on an image.
      docker-load = callBuildPackage ./docker/docker-load.nix {};
      # Create and manage a docker image.
      docker-image = callBuildPackage ./docker/docker-image.nix {};
      # Container execution helpers, currently specific to Nix
      # sandboxed scenarios.
      docker-container = {
        setup = pkgs.callPackage ./docker/container-setup.nix {};
        teardown = pkgs.callPackage ./docker/container-teardown.nix {};
      };
      # Sanity checks for environments executing docker based builds.
      docker-sanity = callBuildPackage ./docker/sanity-check.nix {};

      # RPM metadata extractor, handles parsing and dependency support
      # metadata generation.
      rpm-metadata = callBuildPackage ./rpm/rpm-metadata.nix { inherit (pkgs) rpm; };
      # RPM macros are common macros supplied by and to thar packages.
      rpm-macros = callBuildPackage ./rpm/rpm-macros.nix { inherit (pkgs) rpm; };
      # RPM container is the common build environment for thar packages.
      rpm-container = callBuildPackage ./rpm/rpm-container.nix {};
      # RPM dependency "resolver" (extremely naive and basic) to
      # include RPMs automatically for builds.
      rpm-dependencies = callBuildPackage ./rpm/rpm-dependency-resolver.nix {};
      # RPM derivation builder which is very smart and magical, needs
      # reword to simplify.
      rpmBuilder = callBuildPackage ./rpm/rpm-builder.nix { inherit (pkgs) writeScript; };
      # fetchRpmSources reads a package's metadata and sources to
      # fetch its dependencies automatically.
      fetchRpmSources = callBuildPackage ./rpm/fetch-rpm-sources.nix {};
      # mkMacroPath constructs a path for macro references when
      # provided to rpm cli tools.
      mkMacroPath = paths: builtins.concatStringsSep ":" paths;

      # fetchCargo collects a `cargo vendor' run for building with.
      fetchCargo = callBuildPackage ./rust/fetch-cargo.nix {};
    };

    # buildPackages are the targeted "tharPackages", ie: the rpms.
    buildPackages =  import ../packages { inherit callPackage; };

    # Total closure representing the thar build system and its
    # packages.
    closure = buildSupport // buildPackages;
  in closure;
in
# Construct the closure supported by the set of packages given.
tharClosure nixpkgs'
