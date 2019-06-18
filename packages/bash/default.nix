{ stdenvNoCC }:
stdenvNoCC.mkDerivation {
  name = "bash";
  phases = [ "buildPhase" ];
  buildPhase = ''
  touch $out
  '';
}
