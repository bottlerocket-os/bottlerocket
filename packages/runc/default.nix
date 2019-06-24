{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "runc";
  src = ./.;
}
