{ writeScript }:
writeScript "docker-sandbox-setup" ''
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

containerSetupArgs="-e euid -e egid"
''
