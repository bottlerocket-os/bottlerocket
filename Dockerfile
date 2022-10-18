# syntax=docker/dockerfile:1.4.3
# This Dockerfile has two sections which are used to build rpm.spec packages and to create
# Bottlerocket images, respectively. They are marked as Section 1 and Section 2. buildsys
# uses Section 1 during build-package calls and Section 2 during build-variant calls.
#
# Several commands start with RUN --mount=target=/host, which mounts the docker build
# context (which in practice is the root of the Bottlerocket repository) as a read-only
# filesystem at /host.

ARG SDK
ARG TOOLCHAIN
ARG ARCH
ARG GOARCH

FROM ${SDK} as sdk
FROM --platform=linux/${GOARCH} ${TOOLCHAIN}-${ARCH} as toolchain

############################################################################################
# Section 1: The following build stages are used to build rpm.spec packages

# =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^=
# The experimental cache mount type doesn't expand arguments, so our choices are limited.
# We can either reuse the same cache for all builds, which triggers overlayfs errors if the
# builds run in parallel, or we can use a new cache for each build, which defeats the
# purpose. We work around the limitation by materializing a per-build stage that can be used
# as the source of the cache.
FROM scratch AS cache
ARG PACKAGE
ARG ARCH
ARG TOKEN
# We can't create directories via RUN in a scratch container, so take an existing one.
COPY --chown=1000:1000 --from=sdk /tmp /cache
# Ensure the ARG variables are used in the layer to prevent reuse by other builds.
COPY --chown=1000:1000 .dockerignore /cache/.${PACKAGE}.${ARCH}.${TOKEN}

# =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^=
# Some builds need to modify files in the source directory, for example Rust software using
# build.rs to generate code.  The source directory is mounted in using "--mount=source"
# which is owned by root, and we need to modify it as the builder user.  To get around this,
# we can use a "cache" mount, which we just won't share or reuse.  We mount a cache into the
# location we need to change, and in some cases, set up symlinks so that it looks like a
# normal part of the source tree.  (This is like a tmpfs mount, but cache mounts have more
# flexibility - you can specify a source to set them up beforehand, specify uid/gid, etc.)
# This cache is also variant-specific (in addition to package and arch, like the one above)
# for cases where we need to build differently per variant; the cache will be empty if you
# change BUILDSYS_VARIANT.
FROM scratch AS variantcache
ARG PACKAGE
ARG ARCH
ARG VARIANT
ARG TOKEN
# We can't create directories via RUN in a scratch container, so take an existing one.
COPY --chown=1000:1000 --from=sdk /tmp /variantcache
# Ensure the ARG variables are used in the layer to prevent reuse by other builds.
COPY --chown=1000:1000 .dockerignore /variantcache/.${PACKAGE}.${ARCH}.${VARIANT}.${TOKEN}

# =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^=
# Builds an RPM package from a spec file.
FROM sdk AS rpmbuild
ARG PACKAGE
ARG ARCH
ARG NOCACHE
ARG VARIANT
ARG VARIANT_PLATFORM
ARG VARIANT_RUNTIME
ARG VARIANT_FAMILY
ARG VARIANT_FLAVOR
ARG REPO
ENV VARIANT=${VARIANT}
WORKDIR /home/builder

USER builder
ENV PACKAGE=${PACKAGE} ARCH=${ARCH}
COPY --chown=builder roles/${REPO}.root.json ./rpmbuild/BUILD/root.json
# We attempt to copy `Licenses.toml` and `licenses` for the current build, otherwise
# an empty file and a directory are created so that `bottlerocket-license-tool` will
# fail with a more descriptive error message.
RUN --mount=target=/host \
  ( [ -f /host/Licenses.toml ] \
  && cp /host/Licenses.toml ./rpmbuild/BUILD/ \
  || touch ./rpmbuild/BUILD/Licenses.toml ) \
  && ( [ -d /host/licenses ] \
  && cp -r /host/licenses ./rpmbuild/BUILD/ \
  || mkdir ./rpmbuild/BUILD/licenses )
COPY ./macros/${ARCH} ./macros/shared ./macros/rust ./macros/cargo ./packages/${PACKAGE}/ .
RUN rpmdev-setuptree \
   && cat ${ARCH} shared rust cargo > .rpmmacros \
   && echo "%_cross_variant ${VARIANT}" >> .rpmmacros \
   && echo "%_cross_variant_platform ${VARIANT_PLATFORM}" >> .rpmmacros \
   && echo "%_cross_variant_runtime ${VARIANT_RUNTIME}" >> .rpmmacros \
   && echo "%_cross_variant_family ${VARIANT_FAMILY}" >> .rpmmacros \
   && echo "%_cross_variant_flavor ${VARIANT_FAMILY:-none}" >> .rpmmacros \
   && echo "%_cross_repo_root_json %{_builddir}/root.json" >> .rpmmacros \
   && echo "%_topdir /home/builder/rpmbuild" >> .rpmmacros \
   && rm ${ARCH} shared rust cargo \
   && echo "%bcond_without $(V=${VARIANT_PLATFORM,,}; echo ${V//-/_})_platform" > .bconds \
   && echo "%bcond_without $(V=${VARIANT_RUNTIME,,}; echo ${V//-/_})_runtime" >> .bconds \
   && echo "%bcond_without $(V=${VARIANT_FAMILY,,}; echo ${V//-/_})_family" >> .bconds \
   && echo "%bcond_without $(V=${VARIANT_FLAVOR:-no}; V=${V,,}; echo ${V//-/_})_flavor" >> .bconds \
   && cat .bconds ${PACKAGE}.spec >> rpmbuild/SPECS/${PACKAGE}.spec \
   && find . -maxdepth 1 -not -path '*/\.*' -type f -exec mv {} rpmbuild/SOURCES/ \; \
   && echo ${NOCACHE}

USER root
RUN --mount=target=/host \
    ln -s /host/build/rpms/*.rpm ./rpmbuild/RPMS \
    && createrepo_c \
        -o ./rpmbuild/RPMS \
        -x '*-debuginfo-*.rpm' \
        -x '*-debugsource-*.rpm' \
        --no-database \
        /host/build/rpms \
    && cp .rpmmacros /etc/rpm/macros \
    && dnf -y \
        --disablerepo '*' \
        --repofrompath repo,./rpmbuild/RPMS \
        --enablerepo 'repo' \
        --nogpgcheck \
        builddep rpmbuild/SPECS/${PACKAGE}.spec

# We use the "nocache" writable space to generate code where necessary, like the variant-
# specific models.
USER builder
RUN --mount=source=.cargo,target=/home/builder/.cargo \
    --mount=type=cache,target=/home/builder/.cache,from=cache,source=/cache \
    --mount=type=cache,target=/home/builder/rpmbuild/BUILD/sources/models/src/variant,from=variantcache,source=/variantcache \
    --mount=type=cache,target=/home/builder/rpmbuild/BUILD/sources/logdog/conf/current,from=variantcache,source=/variantcache \
    --mount=source=sources,target=/home/builder/rpmbuild/BUILD/sources \
    rpmbuild -ba --clean \
      --undefine _auto_set_build_flags \
      rpmbuild/SPECS/${PACKAGE}.spec

# =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^=
# Copies RPM packages from the previous stage to their expected location so that buildsys
# can find them and copy them out.
FROM scratch AS package
COPY --from=rpmbuild /home/builder/rpmbuild/RPMS/*/*.rpm /output/

############################################################################################
# Section 2: The following build stages are used to create a Bottlerocket image once all of
# the rpm files have been created by repeatedly using Section 1.

# =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^=
# Creates an RPM repository from packages created in Section 1.
FROM sdk AS repobuild
ARG PACKAGES
ARG ARCH
ARG NOCACHE
WORKDIR /root

USER root
RUN --mount=target=/host \
    mkdir -p /local/rpms /local/migrations /local/archives ./rpmbuild/RPMS \
    && ln -s /host/build/rpms/*.rpm ./rpmbuild/RPMS \
    && find /host/build/rpms/ -maxdepth 1 -type f \
        -name "bottlerocket-${ARCH}-migrations-*.rpm" \
        -not -iname '*debuginfo*' \
        -exec cp '{}' '/local/migrations/' ';' \
    && KERNEL="$(printf "%s\n" ${PACKAGES} | awk '/^kernel-/{print $1}')" \
    && find /host/build/rpms/ -maxdepth 1 -type f \
        -name "bottlerocket-${ARCH}-${KERNEL}-archive-*.rpm" \
        -exec cp '{}' '/local/archives/' ';' \
    && createrepo_c \
        -o ./rpmbuild/RPMS \
        -x '*-debuginfo-*.rpm' \
        -x '*-debugsource-*.rpm' \
        --no-database \
        /host/build/rpms \
    && echo '%_dbpath %{_sharedstatedir}/rpm' >> /etc/rpm/macros \
    && dnf -y \
        --disablerepo '*' \
        --repofrompath repo,./rpmbuild/RPMS \
        --enablerepo 'repo' \
        --nogpgcheck \
        --downloadonly \
        --downloaddir . \
        install $(printf "bottlerocket-${ARCH}-%s\n" ${PACKAGES}) \
    && mv *.rpm /local/rpms \
    && createrepo_c /local/rpms \
    && echo ${NOCACHE}

# =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^=
# Builds a Bottlerocket image.
FROM repobuild as imgbuild
ARG ARCH
ARG VERSION_ID
ARG BUILD_ID
ARG NOCACHE
ARG VARIANT
ARG PRETTY_NAME
ARG IMAGE_NAME
ARG IMAGE_FORMAT
ARG OS_IMAGE_SIZE_GIB
ARG DATA_IMAGE_SIZE_GIB
ARG PARTITION_PLAN
ARG OS_IMAGE_PUBLISH_SIZE_GIB
ARG DATA_IMAGE_PUBLISH_SIZE_GIB
ARG KERNEL_PARAMETERS
ARG GRUB_SET_PRIVATE_VAR
ENV VARIANT=${VARIANT} VERSION_ID=${VERSION_ID} BUILD_ID=${BUILD_ID} \
    PRETTY_NAME=${PRETTY_NAME} IMAGE_NAME=${IMAGE_NAME} \
    KERNEL_PARAMETERS=${KERNEL_PARAMETERS}
WORKDIR /root

USER root
RUN --mount=target=/host \
    /host/tools/rpm2img \
      --package-dir=/local/rpms \
      --output-dir=/local/output \
      --output-fmt="${IMAGE_FORMAT}" \
      --os-image-size-gib="${OS_IMAGE_SIZE_GIB}" \
      --data-image-size-gib="${DATA_IMAGE_SIZE_GIB}" \
      --os-image-publish-size-gib="${OS_IMAGE_PUBLISH_SIZE_GIB}" \
      --data-image-publish-size-gib="${DATA_IMAGE_PUBLISH_SIZE_GIB}" \
      --partition-plan="${PARTITION_PLAN}" \
      --ovf-template="/host/variants/${VARIANT}/template.ovf" \
      ${GRUB_SET_PRIVATE_VAR:+--with-grub-set-private-var=yes} \
    && echo ${NOCACHE}

# =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^=
# Creates an archive of the datastore migrations.
FROM repobuild as migrationbuild
ARG ARCH
ARG VERSION_ID
ARG BUILD_ID
ARG NOCACHE
ARG VARIANT
ENV VARIANT=${VARIANT} VERSION_ID=${VERSION_ID} BUILD_ID=${BUILD_ID}
WORKDIR /root

USER root
RUN --mount=target=/host \
    /host/tools/rpm2migrations \
      --package-dir=/local/migrations \
      --output-dir=/local/output \
    && echo ${NOCACHE}

# =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^=
# Creates an archive of kernel development sources and toolchain.
FROM repobuild as kmodkitbuild
ARG ARCH
ARG VERSION_ID
ARG BUILD_ID
ARG NOCACHE
ARG VARIANT
ENV VARIANT=${VARIANT} VERSION_ID=${VERSION_ID} BUILD_ID=${BUILD_ID}

USER root
COPY --from=toolchain /toolchain /local/toolchain

WORKDIR /tmp
RUN --mount=target=/host \
    /host/tools/rpm2kmodkit \
      --archive-dir=/local/archives \
      --toolchain-dir=/local/toolchain \
      --output-dir=/local/output \
    && echo ${NOCACHE}

# =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^= =^..^=
# Copies the build artifacts (Bottlerocket image files, migrations, and kmod kit) to their
# expected location so that buildsys can find them and copy them out.
FROM scratch AS variant
COPY --from=imgbuild /local/output/. /output/
COPY --from=migrationbuild /local/output/. /output/
COPY --from=kmodkitbuild /local/output/. /output/
