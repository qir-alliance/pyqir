# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

FROM quay.io/pypa/manylinux2014_x86_64 as base-with-rust

ARG USERNAME=runner
ARG USER_UID=1000
ARG USER_GID=${USER_UID}
ARG RUST_VERSION=1.57.0

RUN groupadd --gid ${USER_GID} ${USERNAME}
RUN useradd --uid ${USER_UID} --gid ${USER_GID} -m ${USERNAME}
RUN yum install -y sudo
RUN echo ${USERNAME} ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/${USERNAME}
RUN chmod 0440 /etc/sudoers.d/${USERNAME}

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH} \
    RUST_VERSION=${RUST_VERSION}

RUN curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --no-modify-path --profile minimal --default-toolchain ${RUST_VERSION} -y

RUN chmod -R a+w ${RUSTUP_HOME} ${CARGO_HOME}; \
    rustup --version; \
    cargo --version; \
    rustc --version;

FROM base-with-rust as builder

ARG USERNAME=ciuser

WORKDIR /tmp

USER $USERNAME

# Temporary workaround installing beta for license/notice support
RUN cargo install maturin --git https://github.com/PyO3/maturin --tag v0.12.12-beta.2

FROM base-with-rust

ARG USERNAME=ciuser

# Add all supported python versions
ENV PATH /opt/python/cp36-cp36m/bin:/opt/python/cp37-cp37m/bin:/opt/python/cp38-cp38/bin:/opt/python/cp39-cp39/bin:/opt/python/cp310-cp310/bin:$PATH

RUN python3.6 -m pip install --no-cache-dir cffi \
    && python3.7 -m pip install --no-cache-dir cffi \
    && python3.8 -m pip install --no-cache-dir cffi \
    && python3.9 -m pip install --no-cache-dir cffi \
    && python3.10 -m pip install --no-cache-dir cffi \
    && mkdir /io

COPY --from=builder ${CARGO_HOME}/bin/maturin /usr/bin/maturin

WORKDIR /io

RUN yum install -y libffi-devel ninja-build

ADD https://repo.anaconda.com/miniconda/Miniconda3-py39_4.10.3-Linux-x86_64.sh /tmp/Miniconda3.sh

RUN /bin/bash /tmp/Miniconda3.sh -b -p /usr/local/miniconda3

ENV PATH="/usr/local/miniconda3/bin:${PATH}"

RUN conda init && \
    conda install -y -c conda-forge clang-11 libstdcxx-devel_linux-64 libgcc-devel_linux-64 && \
    cp /usr/local/miniconda3/bin/clang-11 /usr/local/miniconda3/bin/clang++-11

RUN conda run python -m pip install -U tox
