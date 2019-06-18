{ stdenvNoCC, docker-cli }:
{ image }:
stdenvNoCC.mkDerivation {
  name = "docker-load-${image.name}";

  buildInputs = [ docker-cli ];
  # The loader is impure and causes the local system to "do"
  # something.
  allowSubstities = false;
  phases = ["buildPhase"];
  buildPhase = ''
  docker load < ${image}
  ln -s ${image.containerRef} $out
  '';
}
