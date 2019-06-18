{ rpmBuilder, fetchRpmSources }:
rpmBuilder.mkDerivation {
  name = "sdk";
  srcs = [
    ./.
  ];

  rpmInputs = [];
}
