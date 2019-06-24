{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "grub";
  src = ./.;
  rpmInputs = [ sdk ];
}
