{ rpmBuilder, fetchRpmSources }:
rpmBuilder.mkDerivation rec {
  name = "sdk";
  
  src = ./.;
  
  rpmInputs = [];
}
