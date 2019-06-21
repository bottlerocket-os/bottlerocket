{ callPackage }:
rec {
  sdk = callPackage ./sdk {};
  gcc = sdk;
  kernel = callPackage ./kernel {};
  kernel-headers = kernel;
  bash = callPackage ./bash {};
}
