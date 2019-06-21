{ lib, docker-image, base-container-image }:
let
  baseImage = base-container-image.ref;

  # Dependencies for the base image and building
  essentialDeps = lib.escapeShellArgs [ "rpmdevtools" "dnf-plugins-core" "createrepo_c" ];
  # Dependencies needed for the bootstrapping package build.
  bootstrapDeps = lib.escapeShellArgs [ "wget" "python" "perl-ExtUtils-MakeMaker" "bc" "rsync" ];
in
# TODO: make this more pure, where possible.
docker-image.mkImage {
  name = "rpm-build-container-image";

  dockerfile = ''
  FROM ${baseImage}
  RUN dnf upgrade -y
  RUN dnf groupinstall -y "C Development Tools and Libraries"
  RUN dnf install -y ${essentialDeps}
  RUN dnf install -y ${bootstrapDeps}
  '';
}
