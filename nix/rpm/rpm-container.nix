{ lib, docker-image, base-container-image, tharPackages }:
let
  baseImage = base-container-image.ref;

  # buildDeps are the dependencies identified by packages that need to
  # be installed and available in the build environment.
  buildDeps = with lib; let
    # Find packages with dependencies declared.
    havingDeps = attrValues (filterAttrs (n: v: hasAttr "rpmHostInputs" v) tharPackages);
    # Collate and make a list of them.
    packages = unique (naturalSort (flatten (map (d: d.rpmHostInputs) havingDeps)));
  in
    escapeShellArgs packages;

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
  RUN dnf install -y ${buildDeps}
  '';
}
