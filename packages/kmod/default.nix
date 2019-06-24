{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "kmod";
  src = ./.;
}
