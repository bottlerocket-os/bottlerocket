{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "api";
  src = ./.;
  rpmInputs = [ sdk ];
}
