{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "coreutils";
  src = ./.;
  rpmInputs = [ sdk ];
}
