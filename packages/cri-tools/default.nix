{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "cri-tools";
  src = ./.;
}
