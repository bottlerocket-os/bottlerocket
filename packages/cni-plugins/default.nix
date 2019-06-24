{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "cni-plugins";
  src = ./.;
}
