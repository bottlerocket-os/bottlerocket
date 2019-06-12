{ stdenv, ... }:

# Make a singleton derivation that captures the shared macros used in
# rpm building needed at expansion and build time.
#
# At invocation time, the set of macros on disk are collected and
# added to the nix store for use.
stdenv.mkDerivation rec {
  name = "rpm-macros";
  outputs = ["out" "arch"];
  src = ../../macros;
  phases = [ "installPhase" ];
  archMacroRegex = "^%_cross_arch";
  installPhase = ''
  mkdir -p $out $arch
  grep --null --files-with-match    -r '${archMacroRegex}' $src | xargs -0 -t -L1 -I SRC -- cp --no-clobber SRC $arch
  grep --null --files-without-match -r '${archMacroRegex}' $src | xargs -0 -t -L1 -I SRC -- cp --no-clobber SRC $out
  '';
}
