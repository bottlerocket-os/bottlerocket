{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "libkmod";
  src = ./.;
  rpmInputs = [ sdk ];
}
