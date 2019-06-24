{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "util-linux";
  src = ./.;
  rpmInputs = [ sdk ];
}
