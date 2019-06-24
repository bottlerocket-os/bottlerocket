{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "golang";
  src = ./.;
}
