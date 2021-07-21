#!/usr/bin/env bash
set -eo pipefail
# This is a script meant to be used by Github Actions to test out pubsys related cargo-make tasks

required_env() {
  local env="${1:?}"
  local value="${2}"
  if [ -z "${value}" ]; then
    echo "ERROR: Environment variable ${env} is required to be set" >&2
    exit 2
  fi
}

required_env "VARIANT" "${VARIANT}"
required_env "ARCH" "${ARCH}"
required_env "GITHUB_WORKSPACE" "${GITHUB_WORKSPACE}"

cd "${GITHUB_WORKSPACE}"

echo -e "\n=^..^=   =^..^=   =^..^=   Create TUF repository from Bottlerocket build artifacts   =^..^=   =^..^=   =^..^=\n"
cargo make -e BUILDSYS_VARIANT="${VARIANT}" -e BUILDSYS_ARCH="${ARCH}" repo
# refresh-repo will fail if the output directory already exists, so we move it to a known path.
mv build/repos/default build/repos/test

cat <<EOF > Infra.toml
[repo.default]
metadata_base_url = "file://${GITHUB_WORKSPACE}/build/repos/test/latest"
targets_url = "file://${GITHUB_WORKSPACE}/build/repos/test/latest/targets"
EOF

echo -e "\n=^..^=   =^..^=   =^..^=   Test check-repo-expirations   =^..^=   =^..^=   =^..^=\n"
cargo make -e BUILDSYS_VARIANT="${VARIANT}" -e BUILDSYS_ARCH="${ARCH}" check-repo-expirations
echo -e "\n=^..^=   =^..^=   =^..^=   Test validate-repo   =^..^=   =^..^=   =^..^=\n"
cargo make -e BUILDSYS_VARIANT="${VARIANT}" -e BUILDSYS_ARCH="${ARCH}" validate-repo
echo -e "\n=^..^=   =^..^=   =^..^=   Test refresh-repo   =^..^=   =^..^=   =^..^=\n"
cargo make -e BUILDSYS_VARIANT="${VARIANT}" -e BUILDSYS_ARCH="${ARCH}" refresh-repo

# refresh-repo doesn't change the `latest` symlink, so we find the new directory.
REPO_PATH=$(find "${GITHUB_WORKSPACE}"/build/repos/default/ -mindepth 1 -maxdepth 1 -type d)
# refresh-repo doesn't update targets, only metadata, so we reference the existing targets URL.
cat <<EOF > Infra.toml
[repo.default]
metadata_base_url = "file://${REPO_PATH}"
targets_url = "file://${GITHUB_WORKSPACE}/build/repos/test/latest/targets"
EOF

echo -e "\n=^..^=   =^..^=   =^..^=   Try check-repo-expirations after refresh   =^..^=   =^..^=   =^..^=\n"
cargo make -e BUILDSYS_VARIANT="${VARIANT}" -e BUILDSYS_ARCH="${ARCH}" check-repo-expirations
echo -e "\n=^..^=   =^..^=   =^..^=   Try validate-repo after refresh   =^..^=   =^..^=   =^..^=\n"
cargo make -e BUILDSYS_VARIANT="${VARIANT}" -e BUILDSYS_ARCH="${ARCH}" validate-repo
