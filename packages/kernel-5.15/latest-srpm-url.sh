#!/bin/sh
docker run --rm amazonlinux:2 sh -c 'amazon-linux-extras enable kernel-5.15 >/dev/null && yum install -q -y yum-utils && yumdownloader -q --source --urls kernel | grep ^http'
