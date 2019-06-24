{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "libnfnetlink";
  src = ./.;
}
