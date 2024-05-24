#!/bin/sh
cmd="
dnf install -q -y --releasever=latest yum-utils &&
dnf download -q --repofrompath neuron,https://yum.repos.neuron.amazonaws.com --repo=neuron --urls aws-neuronx-dkms
"
docker run --rm amazonlinux:2023 bash -c "${cmd}" \
    | grep '^http' \
    | xargs --max-args=1 --no-run-if-empty realpath --canonicalize-missing --relative-to=. \
    | sed 's_:/_://_'
