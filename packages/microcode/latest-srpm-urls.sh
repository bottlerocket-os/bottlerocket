#!/bin/sh
docker run --rm amazonlinux:2 sh -c 'yum install -q -y yum-utils && yumdownloader -q --source --urls linux-firmware microcode_ctl | grep ^http'
