{ rpmBuilder, runCommand, fetchFromGitHub, sdk, glibc, kernel }:
let
  sourceName = "ncurses-6.1-20180923";
  ncursesSrc = fetchFromGitHub rec {
    name = sourceName;
    owner = "mirror";
    repo = "ncurses";
    rev = "3247e20e3f6d0ba0921604100383d572fbbf507a";
    sha256 = "0001x43v1xpsj3a3b5x25h01j2qr8kyry2qc1bv0mym5vzrv9lxs";
  };
  tarball = runCommand "${sourceName}.tgz" {} ''
  mkdir -p $out
  tar -C  ${ncursesSrc} -zc -f $out/${sourceName}.tgz --transform 's,^,${sourceName}/,' ./
  '';
in
rpmBuilder.mkDerivation {
  name = "ncurses";
  src = ./.;
  srcs = [ tarball ];
  rpmSources = [ ];
  rpmInputs = [ sdk glibc kernel ];
}
