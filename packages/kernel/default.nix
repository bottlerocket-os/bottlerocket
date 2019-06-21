{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "kernel";
  src = ./.;
  rpmInputs = [ sdk ];
  rpmHostInputs = [ "hostname" "openssl-devel" "elfutils-devel" "bc" ];
}
