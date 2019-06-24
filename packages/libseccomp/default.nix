{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "libseccomp";
  src = ./.;
}
