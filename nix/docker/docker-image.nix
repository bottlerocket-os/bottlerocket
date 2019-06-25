{ stdenvNoCC, lib, writeScript, docker-cli }:
let
  mkLoader = { image }: writeScript "loader-${image.name}" ''
  exec 1>&2
    {
      ${docker-cli}/bin/docker inspect --format 'using loaded image: {{.ID}}' \
                                ${lib.fileContents image.containerRef} 2>/dev/null
    } || {
      ${docker-cli}/bin/docker load < ${image.out}
      echo 'loaded image from file'
    }
  '';

  mkImage = { name, dockerfile, ... }@args:
    let
      cleanArgs = removeAttrs args ["name" "buildInputs" "passthru" "propagatedBuildInputs" ];

      passthru = lib.recursiveUpdate {
        docker = {
          loader = mkLoader { image = drv; };
          ref = lib.fileContents drv.containerRef;
        };
      }
        (if args ? passthru then args.passthru else {});

      drv = stdenvNoCC.mkDerivation ({
        inherit name passthru dockerfile;

        outputs = ["out" "containerRef"];
        buildInputs = [ docker-cli ];
        phases = [ "buildPhase" "installPhase" ];
        passAsFile = [ "dockerfile" ];

        buildPhase = ''
        mkdir empty-context
        ref="''${containerRef##*/}"
        ref="''${ref,,}:containerRef"

        awk '/FROM/ { $1=""; print; }' $dockerfile | xargs --no-run-if-empty -L1 -t docker pull
        docker build --build-arg name \
                     --build-arg containerRef \
                     --label containerRef=$containerRef \
                     --network host \
                     --tag "$ref" \
                      --file $dockerfilePath \
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
