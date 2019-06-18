{ docker-image, base-container-image }:
let
  baseImage = base-container-image.ref;
in
# TODO: make this more pure, where possible.
docker-image.mkImage {
  name = "rpm-build-container-image";
  dockerfile = ''
  FROM ${baseImage}
  RUN dnf upgrade -y && \
      dnf groupinstall -y "C Development Tools and Libraries" && \
      dnf install -y rpmdevtools dnf-plugins-core createrepo_c
  '';
}
