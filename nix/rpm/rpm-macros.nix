{ stdenvNoCC, ... }:

# Make a singleton derivation that captures the shared macros used in
# rpm building needed at expansion and build time.
#
# At invocation time, the set of macros on disk are collected and
# added to the nix store for use.
let
  archMacroRegex = "^%_cross_arch";
in
stdenvNoCC.mkDerivation rec {
  name = "rpm-macros";
  
  src = ../../macros;
  
  outputs = ["out" "arches"];
  
  phases = [ "installPhase" ];
  
  installPhase = ''
  set | grep -e 'per.arch'
  mkdir -p $out $per_arch
  grep --null --files-with-match    -r '${archMacroRegex}' $src | xargs -0 -t -L1 -I SRC -- cp --no-clobber SRC $arches
  grep --null --files-without-match -r '${archMacroRegex}' $src | xargs -0 -t -L1 -I SRC -- cp --no-clobber SRC $out
  '';
}
