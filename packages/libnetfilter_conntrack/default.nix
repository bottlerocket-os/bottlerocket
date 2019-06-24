{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "libnetfilter_conntrack";
  src = ./.;
}
