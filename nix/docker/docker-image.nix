{ stdenv, docker-cli }:
let
  mkImage = { name, dockerfile, ... }:
    let
      dockerfileFile = if builtins.isString dockerfile
                       then
                         builtins.toFile "Dockerfile" dockerfile
                       else
                         dockerfile;
    in
    stdenv.mkDerivation {
      inherit name;

      outputs = ["out" "containerRef"];
      buildInputs = [ docker-cli ];
      phases = [ "buildPhase" "installPhase" ];

      buildPhase = ''
  mkdir empty-context
  ref="''${containerRef##*/}"
  ref="''${ref,,}:containerRef"
  docker build --build-arg name \
               --build-arg containerRef \
               --label containerImage=$containerImage \
               --network host \
               --tag "$ref" \
               --file ${dockerfileFile} ./empty-context
  '';
      
      installPhase = ''
  image_id="$(docker images --filter "label=containerImage=$containerImage" --format "{{.ID}}" --no-trunc)"
  docker save "$ref" > $out
  echo "$ref" > $containerRef
  '';
    };
in
{
  inherit mkImage;
}
