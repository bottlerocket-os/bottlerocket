{ rpmBuilder, sdk, glibc, ncurses, kernel }:
rpmBuilder.mkDerivation rec {
  name = "readline";
  src = ./.;
  rpmInputs = [ sdk glibc ncurses kernel ];
}
