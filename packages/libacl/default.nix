{ rpmBuilder, sdk, kernel, glibc, libattr }:
rpmBuilder.mkDerivation rec {
  name = "libacl";
  src = ./.;
  rpmInputs = [ sdk kernel glibc libattr ];
}
