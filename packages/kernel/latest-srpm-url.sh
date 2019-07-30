#!/bin/sh
docker run --rm amazonlinux:2 sh -c 'amazon-linux-extras enable kernel-ng >/dev/null && yum install -q -y yum-utils && yumdownloader -q --source --urls kernel | grep ^http'
