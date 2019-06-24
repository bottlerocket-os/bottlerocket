{ stdenvNoCC, writeScript, git, docker-cli, docker-container,
  cargo, cargo-vendor, rpm-container }:

{ name, cargo-toml, cargo-lock }:
let
  # TODO: Use a different container :)
  containerImage = rpm-container;
  fetchScript = writeScript "cargo-vendor-${name}" ''
  set -ex
  # Catch early exit and run teardown to allow host user to
  # manipulate $out if used.
  trap "originalExit=$?; ${docker-container.teardown}; exit $originalExit;" EXIT
  cd /build

  ln -sv ${cargo-toml} Cargo.toml
  ln -sv ${cargo-lock} Cargo.lock

  # Appease cargo-vendor by stubbing in "sources".
  install -D /dev/null src/lib.rs
  install -D /dev/null src/main.rs

  if [[ ! -r Cargo.lock ]]; then
      echo
      echo "ERROR: The Cargo.lock file doesn't exist at path! (in $(pwd))."
      echo
      exit 1
  fi

  export CARGO_HOME=$(mktemp -d cargo-home.XXX)
  CARGO_CONFIG=$(mktemp cargo-config.XXXX)

  # Replace our temporary path and write to a "template" file for consumers to replace.
  cargo vendor $out/cargo-vendor | sed "s,$out/cargo-vendor,@vendor@,g" > $CARGO_CONFIG
  install -D $CARGO_CONFIG $out/cargo-vendor/.cargo/config.in
  '';
in
stdenvNoCC.mkDerivation {
  inherit name;

  phases = ["unpackPhase"];
  buildInputs = [ git docker-cli cargo cargo-vendor ];

  unpackPhase = ''
  ${containerImage.docker.loader}
  source "${docker-container.setup}"
  # Setup a space for the container to write out to us with the appropriate permissions.
  containerOut="$sandboxBuild/containerOut"
  mkdir -p containerOut

  docker run --rm --entrypoint "/bin/sh" --userns=host --net=host \
                                 --volume "$NIX_STORE:$NIX_STORE:ro" \
                                 --volume "$containerOut:$containerOut" \
                                 --env "out=$containerOut" \
                                 --tmpfs /build:rw,size=8G,mode=1777 \
                                 $containerSetupArgs \
                                 -e src -e srcs -e outputs -e PATH \
                                 ${containerImage.docker.ref} "${fetchScript}"

  mv containerOut/cargo-vendor $out
  '';
}
