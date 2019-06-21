{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "sdk";
  src = ./.;
}
