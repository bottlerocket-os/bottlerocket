{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "kubernetes";
  src = ./.;
}
