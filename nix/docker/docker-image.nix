{ stdenv, docker-cli }:
{ name, dockerfile, ... }:
stdenv.mkDerivation {
  inherit name;

  outputs = ["out" "containerRef"];
  buildInputs = [ docker-cli ];
  phases = [ "buildPhase" "installPhase" ];

  buildPhase = ''
  mkdir empty-context
  docker build --build-arg containerImage \
               --build-arg containerRef \
               --label containerImage=$containerImage \
               --network host \
               --file ${dockerfile} ./empty-context
  '';
  
  installPhase = ''
  ref="''${containerRef##*/}"
  ref="''${ref,,}"
  image_id="$(docker images --filter "label=containerImage=$containerImage" --format "{{.ID}}" --no-trunc)"
  docker tag "$image_id" "$ref"
  docker save "$ref" > $out
  echo "$ref" > $containerRef
  '';
}
