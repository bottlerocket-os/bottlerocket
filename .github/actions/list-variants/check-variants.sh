#!/bin/bash
#
# Collects repo path information to determine what is part of a variant, then
# compares the changed files from the PR to the paths to see which variants are
# actually affected by the changes.
#
# Expects get-changed-files to have been run first with json content of modified
# files in ${HOME}/files.json.

function usage {
    echo "Usage: $0 <filter>"
    echo
    echo -e "\tfilter: boolean indicating whether to return all variants (false) or only those affected by changes (true)"
    echo
    exit 1
}

if [[ $# -lt 1 ]]; then
    usage
fi

filter="$1"

CHANGED_FILES="/tmp/files.txt"
OUTPUT="/tmp/ghoutput"
REPO_ROOT=$(git rev-parse  --show-toplevel)

# Parse the changed file data into something easy for us to parse
jq -r .[] "${HOME}/files.json" > "${CHANGED_FILES}"

# Make sure there are no leftover artifacts
rm -f "${OUTPUT}"

cd "${REPO_ROOT}" || exit
cd variants || exit

aarch="aarch-enemies=$(ls -d */ | cut -d'/' -f 1 | grep -E '(^(metal|vmware)|\-dev$)' | jq -R -s -c 'split("\n")[:-1] | [ .[] | {"variant": ., "arch": "aarch64"}]')"
echo "${aarch}"
echo ${aarch} >> $OUTPUT

cd .. || exit

if [[ "${filter}" != "false" ]]; then
    # Check if any of the changed files are under relevant paths for our variants
    variants=()
    output=$(find ${REPO_ROOT}/variants/* -maxdepth 0 ! -name "target" ! -name "shared" -type d -printf "-v %f ")

    # Generate the set of repo paths that make up each variant
    mkdir -p /tmp/variant-info
    pushd "${REPO_ROOT}/.github/utils/variant-metadata" || exit
    cargo run -- get-source-paths -r "${REPO_ROOT}" ${output} -o /tmp/variant-info
    popd || exit

    output=$(find ${REPO_ROOT}/variants/* -maxdepth 0 ! -name "target" ! -name "shared" -type d -printf "%f\n")

    for variant in ${output}; do
        while read line; do
            if [[ -n $(grep -e ^${line} ${CHANGED_FILES}) ]]; then
              # There are changed files under this path
              variants+=("${variant}")
              break
            fi
        done <"/tmp/variant-info/${variant}"
    done

    vars=$(printf '"%s",' "${variants[@]}")
    vars=${vars::-1}
    output="variants=[${vars}]"
else
    # No need to filter, just get all variants in the repo
    cd variants || exit
    output="variants=$(ls -d */ | cut -d'/' -f 1 | grep -vE '^(shared|target)$' | jq -R -s -c 'split("\n")[:-1]')"
fi

echo "${output}"
echo ${output} >> $OUTPUT
