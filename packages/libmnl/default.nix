{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "libmnl";
  src = ./.;
}
