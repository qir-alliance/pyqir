# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

FROM quay.io/pypa/manylinux2014_x86_64 as builder

ENV PATH /root/.cargo/bin:$PATH

# todo, lock down version
RUN curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

WORKDIR /tmp

# Temporary workaround installing beta for license/notice support
RUN cargo install maturin --git https://github.com/PyO3/maturin --tag v0.12.12-beta.2

FROM quay.io/pypa/manylinux2014_x86_64

ENV PATH /root/.cargo/bin:$PATH
# Add all supported python versions
ENV PATH /opt/python/cp36-cp36m/bin/:/opt/python/cp37-cp37m/bin/:/opt/python/cp38-cp38/bin/:/opt/python/cp39-cp39/bin/:$PATH
# Otherwise `cargo new` errors
ENV USER root

RUN curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && python3 -m pip install --no-cache-dir cffi \
    && mkdir /io

COPY --from=builder /root/.cargo/bin/maturin /usr/bin/maturin

WORKDIR /io

RUN yum install -y libffi-devel ninja-build

ADD https://repo.anaconda.com/miniconda/Miniconda3-py39_4.10.3-Linux-x86_64.sh /tmp/Minoconda.sh

RUN /bin/bash /tmp/Minoconda.sh -b

ENV PATH="/root/miniconda3/bin:${PATH}"

RUN conda init && \
    conda install -y -c conda-forge clang-11 libstdcxx-devel_linux-64 libgcc-devel_linux-64 && \
    cp /root/miniconda3/bin/clang-11 /root/miniconda3/bin/clang++-11

RUN python -m pip install -U tox
