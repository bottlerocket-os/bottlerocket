{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "cni";
  src = ./.;
}
