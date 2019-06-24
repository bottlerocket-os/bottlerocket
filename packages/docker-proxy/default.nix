{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "docker-proxy";
  src = ./.;
}
