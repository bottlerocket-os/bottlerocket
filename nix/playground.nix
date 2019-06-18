with import <nixpkgs> {};
let
  buildScript = pkgs.writeScript "container-buildPhase" ''
  mkdir -p $out
  rpm -qa > $out/rpms.txt
  '';
  containerBuildScript = pkgs.writeScript "container-buildPhase-wrapper" ''
  ${buildScript}
  chown -hR "$euid:$egid" $out
  '';
in
stdenv.mkDerivation {
  name = "test";
  
  buildInputs = with pkgs; [ docker ];
  
  phases = [ "setupPhase" "buildPhase" ];

  setupPhase = ''
  # Record and pass in uid that this should permission its results for
  # (either single user mode nix or the nixbld user's uid). Because
  # both the docker container *and* the nix chroot could be using user
  # namespace remapping, we have to actively choose to not where
  # possible and match up the *ids otherwise.
  #
  # TODO: handle this on other OSes where this may or may not be an issue.
  export euid=$(awk -v "uid=$(id -u)" '{ baseuid=$1; offset=$2; print uid-baseuid+offset; }' /proc/self/uid_map)
  export egid=$(awk -v "gid=$(id -g)" '{ basegid=$1; offset=$2; print gid-basegid+offset; }' /proc/self/gid_map)

  # Resolve ourselves to allow docker to write back to our sandbox.
  sandboxRoot=$(awk '$5 == "/" { print $4; }' /proc/self/mountinfo)
  sandboxBuild=$(awk '$5 == "/build" { print $4; }' /proc/self/mountinfo)
  containerOut="$sandboxBuild/containerOut"

  mkdir -p containerOut
  '';
  
  buildPhase = ''
  docker run --rm --entrypoint "/bin/sh" --userns=host --net=host \
                                         --volume "$NIX_STORE:$NIX_STORE:ro" \
                                         --volume "$containerOut:$containerOut" \
                                         --env "out=$containerOut/out" \
                                         -e euid -e egid \
                                         fedora:latest "${containerBuildScript}"
  mv containerOut/out $out
  '';
}
