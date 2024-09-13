#!/usr/bin/env bash

#
# Common error handling
#

exit_trap_cmds=()

on_exit() {
    exit_trap_cmds+=( "$1" )
}

run_exit_trap_cmds() {
    for cmd in "${exit_trap_cmds[@]}"; do
        eval "${cmd}"
    done
}

trap run_exit_trap_cmds EXIT

warn() {
    >&2 echo "Warning: $*"
}

bail() {
    if [[ $# -gt 0 ]]; then
        >&2 echo "Error: $*"
    fi
    exit 1
}

usage() {
    cat <<EOF
Usage: $0 -r GIT_REPO -v TWOLITER_VERSION -d INSTALL_DIR [-e REUSE_EXISTING] [-b BINARY_INSTALL] [ -s SOURCE_INSTALL ] [-h]

    -r, --repo                    the git or GitHub repository from which to install. For source
                                  install this can be any git repo, including a GitHub. For a binary
                                  installation, this must be a GitHub repository that has binaries
                                  attached to releases.
    -v, --version                 the version (with the v prefix), or the git branch, sha or tag
    -d, --directory               the directory to install twoliter into
    -e, --reuse-existing-install  we will skip installation if we find the correct version installed
    -b, --allow-binary-install    we will try to install a GitHub release-attached binary if the
                                  host we are on is Linux. Takes an expected sha256 sum for the
                                  binary as input.
    -s, --allow-from-source       we will install from source using cargo install pointed to a git
                                  repo and rev when binary install is either not allowed or not
                                  possible
    -k, --skip-version-check      do not check to see if the installed version matches the one that
                                  is requested by the --version argument. twoliter will not be
                                  installed when the binary is present, regardless of what version
                                  it is.
    -h, --help                    show this help text

Example invocation:

    This installs the twoliter program which is needed to build Bottlerocket

    Example, installing binary and reusing-existing (if it exists):

        $0 \\
            -r https://github.com/bottlerocket-os/twoliter \\
            -v v0.1.0 \\
            -d tools/twoliter \\
            -b \\
            -e

    Example, installing from source whether or not it is already installed:

        $0 \\
            -r https://github.com/myfork/twoliter \\
            -v b0482f1 \\
            -d tools/twoliter \\
            -s

EOF
}

usage_error() {
    >&2 usage
    bail "$1"
}


#
# Parse arguments
#

while [[ $# -gt 0 ]]; do
    case $1 in
        -r|--repo)
            shift; repo=$1 ;;
        -v|--version)
            shift; version=$1 ;;
        -d|--directory)
            shift; dir=$1 ;;
        -e|--reuse-existing-install)
            reuse_existing="true" ;;
        -b|--allow-binary-install)
            allow_bin="true"; shift; bin_checksum=$1 ;;
        -s|--allow-from-source)
            from_source="true" ;;
        -k|--skip-version-check)
            skip_version_check="true" ;;
        -h|--help)
            usage; exit 0 ;;
        *)
            usage_error "Invalid option '$1'" ;;
    esac
    shift
done

set -e

workdir="$(mktemp -d)"
on_exit "rm -rf ${workdir}"
mkdir -p "${dir}"

if [ "${reuse_existing}" = "true" ] ; then
   if [ -x "${dir}/twoliter" ] ; then
      if [ "${skip_version_check}" = "true" ]; then
        echo "Twoliter binary found and --skip-version-check is true. Skipping install."
        exit 0
      fi
      version_output="$("${dir}/twoliter" --version)"
      found_version=v$(echo $version_output | awk '{print $2}')
      echo "Found Twoliter ${found_version} installed."
      if [ "${found_version}" = "${version}" ] ; then
         echo "Skipping installation."
         exit 0
      fi
   fi
fi

if [ "${allow_bin}" = "true" ] ; then
   host_arch="$(uname -m)"
   host_arch="${host_arch,,}"
   host_kernel="$(uname -s)"
   host_kernel="${host_kernel,,}"
   case "${host_kernel}-${host_arch}" in
      linux-x86_64 | linux-aarch64)
      echo "Installing Twoliter from binary release."
      twoliter_release="${repo}/releases/download/${version}"
      twoliter_target="${host_arch}-unknown-${host_kernel}-musl"
      cd "${workdir}"
      curl -sSL "${twoliter_release}/twoliter-${twoliter_target}.tar.xz" -o "twoliter.tar.xz"
      echo "Checking binary checksum..."
      sha256sum -c <<< "${bin_checksum} twoliter.tar.xz"
      tar xf twoliter.tar.xz
      mv "./twoliter-${twoliter_target}/twoliter" "${dir}"
      exit 0
      ;;
   *)
      echo "No pre-built binaries available for twoliter ${version}."
      ;;
   esac
else
   echo "Skipping binary installation of twoliter ${version} because --allow-binary-install was not set."
fi

if [ "${from_source}" = "true" ] ; then
   echo "Installing Twoliter version ${version} from source"
   cargo +nightly install \
     -Z bindeps \
     --locked \
     --root "${workdir}" \
     --git "${repo}" \
     --rev "${version}" \
     --bin twoliter \
     --quiet \
     twoliter
   mv "${workdir}/bin/twoliter" "${dir}/twoliter"
   echo "Installed twoliter ${version} from source."
   exit 0
else
   echo "Skipped installing twoliter ${version} from source."
fi


if [ ! -x "${dir}/twoliter" ] ; then
   echo "Could not install twoliter ${version}" >&2
   exit 1
fi
