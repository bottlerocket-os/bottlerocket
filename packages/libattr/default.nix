{ rpmBuilder, sdk, kernel, glibc }:
rpmBuilder.mkDerivation rec {
  name = "libattr";
  src = ./.;
  rpmInputs = [ sdk kernel glibc ];
}
