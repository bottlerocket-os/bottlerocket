{ stdenv, docker-cli }:
{ name, srcImage }:
stdenv.mkDerivation {
  name = "docker-load-${srcImage.name}";
  
  # The loader is impure and causes the local system to "do"
  # something.
  allowSubstities = false;
  phases = ["buildPhase"];
  buildPhase = ''
  docker load < ${srcImage.containerImage}
  ln -s ${srcImage.containerRef} $out
  '';
}
