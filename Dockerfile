# syntax=docker/dockerfile:1.1.3-experimental

ARG SDK
FROM ${SDK} as sdk

# The experimental cache mount type doesn't expand arguments, so our choices are limited.
# We can either reuse the same cache for all builds, which triggers overlayfs errors if
# the builds run in parallel, or we can use a new cache for each build, which defeats the
# purpose. We work around the limitation by materializing a per-build stage that can be
# used as the source of the cache.
FROM scratch AS cache
ARG PACKAGE
ARG ARCH
# We can't create directories via RUN in a scratch container, so take an existing one.
COPY --chown=1000:1000 --from=sdk /tmp /cache
# Ensure the ARG variables are used in the layer to prevent reuse by other builds.
COPY --chown=1000:1000 .dockerignore /cache/.${PACKAGE}.${ARCH}

# Some builds need to modify files in the source directory, for example Rust
# software using build.rs to generate code.  The source directory is mounted in
# using "--mount=source" which is owned by root, and we need to modify it as
# the builder user.  To get around this, we can use a "cache" mount, which we
# just won't share or reuse.  We mount a cache into the location we need to
# change, and in some cases, set up symlinks so that it looks like a normal
# part of the source tree.  (This is like a tmpfs mount, but cache mounts have
# more flexibility - you can specify a source to set them up beforehand,
# specify uid/gid, etc.)  This cache is also variant-specific (in addition to
# package and arch, like the one above) for cases where we need to build
# differently per variant; the cache will be empty if you change
# BUILDSYS_VARIANT.
FROM scratch AS variantcache
ARG PACKAGE
ARG ARCH
ARG VARIANT
# We can't create directories via RUN in a scratch container, so take an existing one.
COPY --chown=1000:1000 --from=sdk /tmp /variantcache
# Ensure the ARG variables are used in the layer to prevent reuse by other builds.
COPY --chown=1000:1000 .dockerignore /variantcache/.${PACKAGE}.${ARCH}.${VARIANT}

FROM sdk AS rpmbuild
ARG PACKAGE
ARG ARCH
ARG NOCACHE
ARG VARIANT
ENV VARIANT=${VARIANT}
WORKDIR /home/builder

USER builder
ENV PACKAGE=${PACKAGE} ARCH=${ARCH}
COPY ./macros/${ARCH} ./macros/shared ./macros/rust ./macros/cargo ./packages/${PACKAGE}/* .
RUN rpmdev-setuptree \
   && cat ${ARCH} shared rust cargo > .rpmmacros \
   && rm ${ARCH} shared rust cargo \
   && mv *.spec rpmbuild/SPECS \
   && find . -maxdepth 1 -not -path '*/\.*' -type f -exec mv {} rpmbuild/SOURCES/ \; \
   && echo ${NOCACHE}

USER root
RUN --mount=target=/host \
    ln -s /host/build/*.rpm ./rpmbuild/RPMS \
    && createrepo_c \
        -o ./rpmbuild/RPMS \
        -x '*-debuginfo-*.rpm' \
        -x '*-debugsource-*.rpm' \
        --no-database \
        /host/build \
    && cp .rpmmacros /etc/rpm/macros \
    && dnf -y \
        --disablerepo '*' \
        --repofrompath repo,./rpmbuild/RPMS \
        --enablerepo 'repo' \
        --nogpgcheck \
        builddep rpmbuild/SPECS/${PACKAGE}.spec

# We use the "nocache" writable space to generate code where necessary, like
# the variant-specific models.
USER builder
RUN --mount=source=.cargo,target=/home/builder/.cargo \
    --mount=type=cache,target=/home/builder/.cache,from=cache,source=/cache \
    --mount=type=cache,target=/home/builder/rpmbuild/BUILD/workspaces/models/src/variant,from=variantcache,source=/variantcache \
    --mount=source=workspaces,target=/home/builder/rpmbuild/BUILD/workspaces \
    rpmbuild -ba --clean rpmbuild/SPECS/${PACKAGE}.spec

FROM scratch AS package
COPY --from=rpmbuild /home/builder/rpmbuild/RPMS/*/*.rpm /output/

FROM sdk AS repobuild
ARG PACKAGES
ARG ARCH
ARG NOCACHE
WORKDIR /root

USER root
RUN --mount=target=/host \
    mkdir -p /local/rpms /local/migrations ./rpmbuild/RPMS \
    && ln -s /host/build/*.rpm ./rpmbuild/RPMS \
    && cp /host/build/thar-${ARCH}-migrations-*.rpm /local/migrations \
    && createrepo_c \
        -o ./rpmbuild/RPMS \
        -x '*-debuginfo-*.rpm' \
        -x '*-debugsource-*.rpm' \
        --no-database \
        /host/build \
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

FROM repobuild as imgbuild
ARG ARCH
ARG VERSION_ID
ARG BUILD_ID
ARG NOCACHE
ARG VARIANT
ENV VARIANT=${VARIANT} VERSION_ID=${VERSION_ID} BUILD_ID=${BUILD_ID}
WORKDIR /root

USER root
RUN --mount=target=/host \
    /host/tools/rpm2img \
      --package-dir=/local/rpms \
      --output-dir=/local/output \
    && echo ${NOCACHE}

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

FROM scratch AS variant
COPY --from=imgbuild /local/output/* /output/
COPY --from=migrationbuild /local/output/* /output/
