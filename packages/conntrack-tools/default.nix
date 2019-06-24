{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "conntrack-tools";
  src = ./.;
}
