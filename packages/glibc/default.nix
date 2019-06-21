{ rpmBuilder, sdk, kernel-headers }:
rpmBuilder.mkDerivation rec {
  name = "glibc";
  src = ./.;
  rpmInputs = [ sdk kernel-headers ];
}
