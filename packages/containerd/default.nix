{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "containerd";
  src = ./.;
}
