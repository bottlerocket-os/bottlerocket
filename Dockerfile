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

FROM sdk AS rpmbuild
ARG PACKAGE
ARG ARCH
ARG NOCACHE
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

USER builder
RUN --mount=source=.cargo,target=/home/builder/.cargo \
    --mount=type=cache,target=/home/builder/.cache,from=cache,source=/cache \
    --mount=source=workspaces,target=/home/builder/rpmbuild/BUILD/workspaces \
    rpmbuild -ba --clean rpmbuild/SPECS/${PACKAGE}.spec

FROM scratch AS rpm
COPY --from=rpmbuild /home/builder/rpmbuild/RPMS/*/*.rpm /output/

FROM sdk AS imgbuild
ARG PACKAGES
ARG ARCH
ARG NOCACHE
ARG FLAVOR
ENV FLAVOR=${FLAVOR}
WORKDIR /root

USER root
RUN --mount=target=/host \
    mkdir -p /local/rpms ./rpmbuild/RPMS \
    && ln -s /host/build/*.rpm ./rpmbuild/RPMS \
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
        install $(printf "thar-${ARCH}-%s\n" ${PACKAGES}) \
    && mv *.rpm /local/rpms \
    && createrepo_c /local/rpms \
    && /host/tools/rpm2img \
        --package-dir=/local/rpms \
        --output-dir=/local/output \
    && echo ${NOCACHE}

FROM scratch AS image
COPY --from=imgbuild /local/output/* /output/
