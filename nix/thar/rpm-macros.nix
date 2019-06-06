{ stdenv, ... }:
stdenv.mkDerivation rec {
  name = "thar-rpm-macros";
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
