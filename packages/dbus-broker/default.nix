{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "dbus-broker";
  src = ./.;
}
