{ rpmBuilder, sdk }:
rpmBuilder.mkDerivation rec {
  name = "kernel";
  src = ./.;
  rpmInputs = [ sdk ];

  useHostNetwork = true;
  allowBuilddepDownload = true;
}
