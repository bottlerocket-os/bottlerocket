{ stdenv, lib, writeScript, docker-cli }:
let
  mkLoader = { image }: writeScript "loader-${image.name}" ''
  exec 1>&2
  ${docker-cli}/bin/docker inspect ${lib.fileContents image.containerRef} || \
  ${docker-cli}/bin/docker load ${image}
  '';
  mkImage = { name, dockerfile, ... }@args:
    let
      cleanArgs = removeAttrs args ["name" "passthru" "dockerfile"];
      pthru = if args ? pthru then pthru else {};
      dockerfileFile = if builtins.isString dockerfile
                       then
                         builtins.toFile "Dockerfile" dockerfile
                       else
                         dockerfile;
      passthru = let
        image = drv;
        docker = {
          loader = mkLoader { inherit image; };
          ref = lib.fileContents drv.containerRef;
        }; in {
          inherit docker;
        };
      drv = stdenv.mkDerivation ({
        inherit name passthru;

        outputs = ["out" "containerRef"];
        buildInputs = [ docker-cli ];
        phases = [ "buildPhase" "installPhase" ];

        buildPhase = ''
        mkdir empty-context
        ref="''${containerRef##*/}"
        ref="''${ref,,}:containerRef"

        awk '/FROM/ { $1=""; print; }' ${dockerfileFile} | xargs --no-run-if-empty -L1 -t docker pull
        docker build --build-arg name \
                     --build-arg containerRef \
                     --label containerRef=$containerRef \
                     --network host \
                     --tag "$ref" \
                      --file ${dockerfileFile} \
                     ./empty-context
        '';

        installPhase = ''
        image_id="$(docker images --filter "label=containerImage=$containerImage" --format "{{.ID}}" --no-trunc)"
        docker save "$ref" > $out
        echo "$ref" > $containerRef
        '';
      } // cleanArgs);
    in
      drv;
  in
{
  inherit mkImage mkLoader;
}
