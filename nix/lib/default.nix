{ nixpkgs, ... }:
# This is a watered down closure evaluator from Nixpkgs that does the
# needful here.
#
# https://github.com/NixOS/nixpkgs/blob/eb9a9bb22aa309e857ce7cffb158e80b35869980/lib/customisation.nix#L87-L125
{
  inherit (nixpkgs.lib) callPackageWith callPackagesWith;
  # /* Call the package function in the file `fn' with the required
  #   arguments automatically.  The function is called with the
  #   arguments `args', but any missing arguments are obtained from
  #   `autoArgs'.  This function is intended to be partially
  #   parameterised, e.g.,
  #     callPackage = callPackageWith pkgs;
  #     pkgs = {
  #       libfoo = callPackage ./foo.nix { };
  #       libbar = callPackage ./bar.nix { };
  #     };
  #   If the `libbar' function expects an argument named `libfoo', it is
  #   automatically passed as an argument.  Overrides or missing
  #   arguments can be supplied in `args', e.g.
  #     libbar = callPackage ./bar.nix {
  #       libfoo = null;
  #       enableX11 = true;
  #     };
  # */
  # callPackageWith = autoArgs: fn: args:
  #   let
  #     f = if builtins.isFunction fn then fn else import fn;
  #     auto = builtins.intersectAttrs (builtins.functionArgs f) autoArgs;
  #   in (auto // args);

  # /* Like callPackage, but for a function that returns an attribute
  #    set of derivations. The override function is added to the
  #    individual attributes. */
  # callPackagesWith = autoArgs: fn: args:
  #   let
  #     f = if builtins.isFunction fn then fn else import fn;
  #     auto = builtins.intersectAttrs (builtins.functionArgs f) autoArgs;
  #     origArgs = auto // args;
  #     pkgs = f origArgs;
  #   in pkgs;
}
