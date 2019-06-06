{ nixpkgs }:
{
  fedora = nixpkgs.callPackage ./fedora.nix {};
}
