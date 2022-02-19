# syntax=docker/dockerfile:1.2

# about ARGs:
# @see https://docs.docker.com/engine/reference/builder/#understand-how-arg-and-from-interact
ARG DU_BUST=1

# use it only if you really need to squeeze the bytes
# note: alpine base already comes with ~5.61 MB (alpine 3.11)
ARG STRIP=0

# if you want more information on panics
ARG RUST_BACKTRACE=0

# ARG BUILD_BASE_IMAGE=ghcr.io/asaaki/rust-musl-cross:x86_64-musl
# ARG BUILD_BASE_IMAGE=ekidd/rust-musl-builder:latest
ARG BUILD_BASE_IMAGE=ghcr.io/asaaki/rust-musl-builder:latest
ARG RUN_BASE_IMAGE=alpine:3.15

#########################
##### builder layer #####
#########################

FROM ${BUILD_BASE_IMAGE} AS builder
# base image might have changed it, so let's enforce root to be able to do system changes
USER root

ENV BUILD_CACHE_BUSTER="2021-10-03T00:00:00"
ENV DEB_PACKAGES="ca-certificates curl file git make patch wget xz-utils"

# @see https://github.com/moby/buildkit/blob/master/frontend/dockerfile/docs/experimental.md#example-cache-apt-packages
RUN rm -f /etc/apt/apt.conf.d/docker-clean && \
    echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache

RUN \
  --mount=type=cache,target=/var/cache/apt \
  --mount=type=cache,target=/var/lib/apt \
    echo "===== Build environment =====" && \
    uname -a && \
    echo "===== Dependencies =====" && \
    apt-get update && \
    apt-get install -y --no-install-recommends $DEB_PACKAGES && \
    echo "===== Toolchain =====" && \
    rustup --version && \
    cargo --version && \
    rustc --version && \
    echo "Rust builder image done."

#######################
##### build layer #####
#######################

FROM builder as build
WORKDIR /app
# ENV RUSTFLAGS="-C target-feature=-crt-static"

# (UN)COMMENT what you (don't) need
COPY Cargo.* build.rs /app/
COPY src /app/src

RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/app/target \
    cargo fetch && \
    cargo install --root /app --target=x86_64-unknown-linux-musl --path .

# remove debug symbols

ARG STRIP
RUN [ "${STRIP}" = "1" ] && \
    ( echo "Stripping debug symbols ..."; \
      cd bin && find -executable -type f -exec strip {} + \
    ) || \
    echo "No stripping enabled"

ARG DU_BUST
RUN du -h bin/*

######################
##### base layer #####
######################

FROM ${RUN_BASE_IMAGE} as base

# Why I still add an init process executable:
# @see https://stackoverflow.com/a/51172104/653173
RUN apk update --no-cache && \
    apk upgrade --no-cache && \
    apk add --no-cache tini
ENTRYPOINT ["/sbin/tini", "--"]
ENV TINI_SUBREAPER=1

WORKDIR /app
CMD ["/bin/sh"]

####################
##### run layer ####
####################

# This is why we do not want to use 'FROM scratch',
# otherwise the user within the container would be still root

FROM base as run
ARG USER_ID=1001
RUN addgroup -g ${USER_ID} appuser && \
    adduser  -u ${USER_ID} -G appuser -H -D appuser
USER ${USER_ID}

#######################
##### final image #####
#######################

FROM run as production

ARG RUST_BACKTRACE
ENV RUST_BACKTRACE=${RUST_BACKTRACE}

COPY --from=build --chown=appuser:appuser /app/bin/* /app
COPY static /app/static

# explicitly provide the CMD via docker run, docker-compose, or kubernetes manifest
