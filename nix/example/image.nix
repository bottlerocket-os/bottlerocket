{ stdenv, docker-image }:
docker-image {
  name = "example-image";
  dockerfile = builtins.toFile "Dockerfile" ''
  FROM fedora:latest
  RUN dnf groupinstall -y "Development Tools" && \
      dnf install -y yum-utils rpmdevtools
  '';
}
