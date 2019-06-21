{ callPackage }:
let
  tharPackages = rec {
    inherit tharPackages;

    sdk = callPackage ./sdk {};
    gcc = sdk;

    glibc = callPackage ./glibc {};

    kernel = callPackage ./kernel {};
    kernel-headers = kernel;

    ncurses = callPackage ./ncurses {};
    readline = callPackage ./readline {};
    libattr = callPackage ./libattr {};
    libacl = callPackage ./libacl {};
    bash = callPackage ./bash {};
    signpost = callPackage ./signpost {};
    rust = callPackage ./rust {};
  };
in
tharPackages
