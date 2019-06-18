#!/usr/bin/env bash

package_listing=$(printf "  %s = callPackage ./%s {};\n" \
	$(find ./packages/ -mindepth 2 -name default.nix \
	      | sed 's,./packages/,,' \
              | awk -F'/' '{ print $(NF-1), $0 }'))


cat <<EOF
{ callPackage }:
{
$package_listing  
}
EOF
