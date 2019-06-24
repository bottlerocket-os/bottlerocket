{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "kernel";
  src = ./.;
}
