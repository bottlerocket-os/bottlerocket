{ callPackage }:
let
  tharPackages = rec {
    inherit tharPackages;

    sdk = callPackage ./sdk {};
    gcc = sdk;
    kernel = callPackage ./kernel {};
    kernel-headers = kernel;
    bash = callPackage ./bash {};
    signpost = callPackage ./signpost {};
    rust = callPackage ./rust {};
  };
in
tharPackages
