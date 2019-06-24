{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "socat";
  src = ./.;
}
