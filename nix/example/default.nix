{ callPackage }:
{
  rpmbuild = callPackage ./rpmbuild.nix {};
  image = callPackage ./image.nix {};
}
