{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation {
  name = "bash";
  src = ./.;
  rpmInputs = [ sdk ];
}
