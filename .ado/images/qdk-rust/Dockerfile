ARG BASE_REGISTRY=qdkacr-cfgqhwddajdaebhg.azurecr.io
ARG BASE_TAG=latest

FROM ${BASE_REGISTRY}/qdk-llvm:${BASE_TAG}

ARG MSRUSTUP_FEED_URL
ARG MSRUSTUP_ACCESS_TOKEN
ARG RUST_TOOLCHAIN=ms-prod-1.95

# See https://eng.ms/docs/more/languages-at-microsoft/rust/articles/gettingstarted/install/ci/container-images.html
ENV CARGO_HOME=/opt/cargo
ENV MSRUSTUP_HOME=/opt/msrustup
ENV MSRUSTUP_TOOLCHAIN=${RUST_TOOLCHAIN}
ENV PATH="${MSRUSTUP_HOME}/multiplexers/bin:${CARGO_HOME}/bin:${PATH}"

COPY install-msrustup.sh /tmp/install-msrustup.sh

RUN export MSRUSTUP_ACCESS_TOKEN MSRUSTUP_FEED_URL && \
    pushd /tmp && bash ./install-msrustup.sh && \
    popd && /tmp/msrustup toolchain install ${RUST_TOOLCHAIN} && \
    rm -f /tmp/msrustup /tmp/install-msrustup.sh

WORKDIR /work

RUN echo "Cargo version: $(cargo --version)" \
    && echo "Rust version: $(rustc --version)"
