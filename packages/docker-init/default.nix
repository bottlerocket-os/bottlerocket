{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "docker-init";
  src = ./.;
}
