# syntax=docker/dockerfile:1.2
ARG BASE_REGISTRY=qdkacr-cfgqhwddajdaebhg.azurecr.io
ARG BASE_TAG=latest

FROM ${BASE_REGISTRY}/qdk-llvm:${BASE_TAG}

ARG MSRUSTUP_FEED_URL
ARG RUST_TOOLCHAIN=ms-prod-1.95

ENV CARGO_HOME=/opt/cargo
ENV MSRUSTUP_HOME=/opt/msrustup
ENV PATH="${MSRUSTUP_HOME}/multiplexers/bin:${CARGO_HOME}/bin:${PATH}"

COPY install-msrustup.sh /tmp/install-msrustup.sh

RUN --mount=type=secret,id=msrustup_token \
    MSRUSTUP_ACCESS_TOKEN="$(cat /run/secrets/msrustup_token)" && \
    export MSRUSTUP_ACCESS_TOKEN MSRUSTUP_FEED_URL && \
    pushd /tmp && bash ./install-msrustup.sh && \
    popd && /tmp/msrustup toolchain install && \
    rm -f /tmp/msrustup /tmp/install-msrustup.sh

WORKDIR /work

RUN echo "Cargo version: $(cargo --version)" \
    && echo "Rust version: $(rustc --version)"
