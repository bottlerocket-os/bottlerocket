{ callPackage }:
{
  bash = callPackage ./bash/default.nix {};
  sdk = callPackage ./sdk/default.nix {};  
}
