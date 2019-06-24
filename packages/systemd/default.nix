{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "systemd";
  src = ./.;
  rpmInputs = [ sdk ];
}
