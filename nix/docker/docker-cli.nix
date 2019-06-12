{ stdenv, docker }:
stdenv.mkDerivation {
  name = "docker-cli";
  phases = ["installPhase"];
  installPhase = "install -D -m 555 ${docker}/libexec/docker/docker $out/bin/docker";
}
