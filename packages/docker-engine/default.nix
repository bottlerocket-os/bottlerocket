{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "docker-engine";
  src = ./.;
}
