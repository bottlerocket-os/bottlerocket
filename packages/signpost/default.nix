{ rpmBuilder, fetchcargo, glibc, rust }:
let
  project = ../../workspaces/signpost;
  # TODO: need to copy this in the build and setup the .cargo/config
  # with the appropriate contents. See:
  #
  # - https://github.com/NixOS/nixpkgs/blob/master/pkgs/build-support/rust/fetchcargo-default-config.toml
  # - https://github.com/NixOS/nixpkgs/blob/f3282c8d1e0ce6ba5d9f6aeddcfad51d879c7a4a/pkgs/build-support/rust/default.nix#L33-L41
  #
  cargoDeps = fetchcargo {
    src = project;
    sha256 = "1mip2jbhyr14l3qsk2n9mcazrdd9sj9m6f6saccr5937h5i934id";
  };
in
# rpmBuilder.mkDerivation rec {
#   name = "signpost";
#   src = ./.;
#   srcs = [ project cargo ];
#   rpmInputs = [  ];
# }
cargoDeps
