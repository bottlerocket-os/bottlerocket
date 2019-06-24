{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "libcap";
  src = ./.;
  rpmInputs = [ sdk ];
}
