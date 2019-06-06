{ stdenv, docker-cli, ... }:
stdenv.mkDerivation {
  name = "playground";
  buildInputs = [ docker-cli ];
  buildPhase = ''
  docker run fedora:latest curl checkip.amazonaws.com > $out
  '';
  phases = [ "buildPhase" ];
}
