{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "docker-cli";
  src = ./.;
}
