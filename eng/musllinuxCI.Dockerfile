# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

# As of 18 April 2022
# 3.14 has llvm/clang 11.1.0
# 3.15 has llvm/clang 12.0.1
# edge has llvm/13.0.1

FROM python:3.6-alpine3.14 as base

ARG RUST_VERSION=1.57.0

WORKDIR /oi

FROM base as base-with-rust

ARG RUST_VERSION=1.57.0

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH} \
    RUST_VERSION=${RUST_VERSION}

# install build base deps
RUN apk add build-base

# Install rust deps
RUN apk add libcrypto1.1 libcurl libgcc libssl1.1 musl zlib

# install specific rust toolchain
RUN apk add rustup
RUN rustup-init --no-modify-path --profile minimal --default-toolchain ${RUST_VERSION} -y

RUN chmod -R a+w ${RUSTUP_HOME} ${CARGO_HOME}; \
    rustup --version; \
    cargo --version; \
    rustc --version;

FROM base-with-rust as builder

# Temporary workaround installing beta for license/notice support
RUN cargo install maturin --git https://github.com/PyO3/maturin --tag v0.12.12

FROM base-with-rust

RUN apk add samurai

RUN apk add libffi-dev py3-pip cmake git ccache clang-dev

# install the requirements
RUN apk add --no-cache \
    ca-certificates \
    less \
    ncurses-terminfo-base \
    krb5-libs \
    libgcc \
    libintl \
    libssl1.1 \
    libstdc++ \
    tzdata \
    userspace-rcu \
    zlib \
    icu-libs \
    curl

RUN apk -X https://dl-cdn.alpinelinux.org/alpine/edge/main add --no-cache \
    lttng-ust

# Download the powershell '.tar.gz' archive
RUN curl -L https://github.com/PowerShell/PowerShell/releases/download/v7.2.2/powershell-7.2.2-linux-alpine-x64.tar.gz -o /tmp/powershell.tar.gz

# Create the target folder where powershell will be placed
RUN mkdir -p /opt/microsoft/powershell/7

# Expand powershell to the target folder
RUN tar zxf /tmp/powershell.tar.gz -C /opt/microsoft/powershell/7

# Set execute permissions
RUN chmod +x /opt/microsoft/powershell/7/pwsh

# Create the symbolic link that points to pwsh
RUN ln -s /opt/microsoft/powershell/7/pwsh /usr/bin/pwsh

RUN apk add patchelf

RUN python -m pip install -U pip
RUN python -m pip install --no-cache-dir cffi

COPY --from=builder ${CARGO_HOME}/bin/maturin /usr/bin/maturin

RUN python -m pip install -U tox
