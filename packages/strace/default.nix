{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "strace";
  src = ./.;
}
