{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "sdk";
  src = ./.;
  rpmHostInputs = [ "bc" "perl-ExtUtils-MakeMaker" "python" "rsync" "wget" ];
}
