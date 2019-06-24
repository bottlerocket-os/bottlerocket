{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "iptables";
  src = ./.;
}
