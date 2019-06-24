{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "libnetfilter_queue";
  src = ./.;
}
