{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "release";
  src = ./.;
  rpmInputs = [ sdk ];
}
