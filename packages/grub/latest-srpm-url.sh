#!/bin/sh
cmd='dnf install -q -y --releasever=latest yum-utils && yumdownloader -q --releasever=latest --source --urls grub2'
docker run --rm amazonlinux:2023 sh -c "${cmd}" \
    | grep '^http' \
    | xargs --max-args=1 --no-run-if-empty realpath --canonicalize-missing --relative-to=. \
    | sed 's_:/_://_'
