{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "filesystem";
  src = ./.;
  rpmInputs = [ sdk ];
}
