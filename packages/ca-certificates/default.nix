{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "ca-certificates";
  src = ./.;
}
