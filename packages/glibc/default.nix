{ rpmBuilder, sdk, kernel }:
rpmBuilder.mkDerivation rec {
  name = "glibc";
  src = ./.;
  rpmInputs = [ sdk kernel ];
}
