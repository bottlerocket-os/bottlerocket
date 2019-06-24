{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "libxcrypt";
  src = ./.;
  rpmInputs = [ sdk ];
}
