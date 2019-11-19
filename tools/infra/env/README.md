# CI Builder Environment

This directory provides the working environment within the context of a CI build.
The `lib/` directory contains bootstrap and support resources that are not otherwise not directly executed.
The `bin/` directory contains scripts and executables that are directly used in the build process. 
Scripts named `setup-$NAME` are intended to be sourced into the build environment; scripts sourced into the build environment must be compatible with the bourne shell - `/bin/sh`.

