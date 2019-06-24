{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "libnftnl";
  src = ./.;
}
