# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

FROM quay.io/pypa/manylinux2014_x86_64 as builder

ENV PATH /root/.cargo/bin:$PATH

RUN curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.57.0 -y

WORKDIR /tmp
RUN curl -SsL https://github.com/PyO3/maturin/archive/refs/tags/v0.11.1.tar.gz -o v0.11.1.tar.gz && \
    tar -xz -f ./v0.11.1.tar.gz

RUN mv ./maturin-0.11.1 /maturin

# Manually update the timestamps as ADD keeps the local timestamps and cargo would then believe the cache is fresh
RUN touch /maturin/src/lib.rs /maturin/src/main.rs

RUN cargo rustc --bin maturin --manifest-path /maturin/Cargo.toml --release -- -C link-arg=-s \
    && mv /maturin/target/release/maturin /usr/bin/maturin \
    && rm -rf /maturin

FROM quay.io/pypa/manylinux2014_x86_64

ENV PATH /root/.cargo/bin:$PATH
# Add all supported python versions
ENV PATH /opt/python/cp36-cp36m/bin/:/opt/python/cp37-cp37m/bin/:/opt/python/cp38-cp38/bin/:/opt/python/cp39-cp39/bin/:$PATH
# Otherwise `cargo new` errors
ENV USER root

RUN curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.57.0 -y \
    && python3.6 -m pip install --no-cache-dir cffi \
    && python3.7 -m pip install --no-cache-dir cffi \
    && python3.8 -m pip install --no-cache-dir cffi \
    && python3.9 -m pip install --no-cache-dir cffi \
    && mkdir /io

COPY --from=builder /usr/bin/maturin /usr/bin/maturin

WORKDIR /io

RUN yum install -y libffi-devel ninja-build

ADD https://repo.anaconda.com/miniconda/Miniconda3-py39_4.10.3-Linux-x86_64.sh /tmp/Miniconda3.sh

RUN /bin/bash /tmp/Miniconda3.sh -b

ENV PATH="/root/miniconda3/bin:${PATH}"

RUN conda init && \
    conda install -y -c conda-forge clang-11 libstdcxx-devel_linux-64 libgcc-devel_linux-64 && \
    cp /root/miniconda3/bin/clang-11 /root/miniconda3/bin/clang++-11

RUN python -m pip install -U tox
