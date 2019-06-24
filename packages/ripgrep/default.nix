{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "ripgrep";
  src = ./.;
  rpmInputs = [ sdk ];
}
