{ stdenvNoCC, docker-cli }:
{ image }:
stdenvNoCC.mkDerivation {
  name = "docker-load-${image.name}";

  buildInputs = [ docker-cli ];
  allowSubstitutes = false;
  preferLocalBuild = false;
  phases = ["buildPhase"];
  buildPhase = ''
  docker load < ${image}
  ln -s ${image.containerRef} $out
  '';
}
