{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "libnetfilter_cttimeout";
  src = ./.;
}
