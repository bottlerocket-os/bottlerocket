{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "libexpat";
  src = ./.;
}
