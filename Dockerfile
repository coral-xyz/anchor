FROM rust:1-alpine3.17 AS builder

# Workaround for OOM issue in libgit2:
# https://github.com/docker/build-push-action/issues/621
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN apk add \
    --no-cache \
    git \
    make \
    musl-dev \
    perl

COPY . /usr/local/src/anchor
WORKDIR /usr/local/src/anchor
RUN cargo install \
    --path cli \
    light-anchor-cli

FROM alpine:3.17

COPY --from=builder /usr/local/cargo/bin/light-anchor /usr/local/bin/light-anchor

ENTRYPOINT ["/usr/local/bin/light-anchor"]
