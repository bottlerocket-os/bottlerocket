{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "rust";
  src = ./.;
}
