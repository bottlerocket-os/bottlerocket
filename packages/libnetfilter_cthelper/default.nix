{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "libnetfilter_cthelper";
  src = ./.;
}
