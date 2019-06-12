{ stdenv, docker-image }:
docker-image {
  name = "example-image";
  dockerfile = builtins.toFile "Dockerfile" ''
  FROM fedora:latest
  RUN dnf install -y procps-ng
  '';
}
