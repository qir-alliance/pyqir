ARG BASE_REGISTRY=qdkacr-cfgqhwddajdaebhg.azurecr.io
ARG LLVM_INSTALL_PREFIX=/opt/llvm-20
ARG PYTHON_VERSION=3.11

FROM ${BASE_REGISTRY}/ubi8/ubi:8.10 AS toolchain

ARG PYTHON_VERSION

RUN dnf install -y \
        python${PYTHON_VERSION} \
        python${PYTHON_VERSION}-pip \
        git \
        gcc \
        gcc-c++ \
        make \
        cmake \
        ninja-build \
        curl \
        tar \
        gzip \
        xz \
    && dnf clean all \
    && rm -rf /var/cache/dnf

FROM toolchain AS builder

ARG LLVM_INSTALL_PREFIX
ARG LLVM_ARCHIVE=llvm-project-llvmorg-20.1.8.tar.gz

COPY ${LLVM_ARCHIVE} /tmp/llvm-project.tar.gz

RUN mkdir /tmp/llvm-project \
    && tar -xzf /tmp/llvm-project.tar.gz \
        --strip-components=1 \
        -C /tmp/llvm-project \
    && cmake \
        -S /tmp/llvm-project/llvm \
        -B /tmp/llvm-build \
        -G Ninja \
        -DCMAKE_BUILD_TYPE=MinSizeRel \
        -DCMAKE_INSTALL_PREFIX="${LLVM_INSTALL_PREFIX}" \
        -DLLVM_TARGETS_TO_BUILD=Native \
        -DLLVM_INCLUDE_TESTS=OFF \
        -DLLVM_INCLUDE_BENCHMARKS=OFF \
        -DLLVM_INCLUDE_EXAMPLES=OFF \
        -DLLVM_INCLUDE_DOCS=OFF \
        -DLLVM_ENABLE_LIBXML2=OFF \
        -DLLVM_ENABLE_LIBEDIT=OFF \
        -DLLVM_ENABLE_LIBPFM=OFF \
        -DLLVM_ENABLE_BINDINGS=OFF \
        -DLLVM_ENABLE_ZLIB=OFF \
        -DLLVM_ENABLE_ZSTD=OFF \
    && cmake --build /tmp/llvm-build \
        --target install \
        --parallel 2

FROM toolchain

ARG LLVM_INSTALL_PREFIX

COPY --from=builder ${LLVM_INSTALL_PREFIX}/ ${LLVM_INSTALL_PREFIX}/

ENV PATH="${LLVM_INSTALL_PREFIX}/bin:${PATH}"
ENV LLVM_SYS_201_PREFIX="${LLVM_INSTALL_PREFIX}"

WORKDIR /work

RUN test "$(llvm-config --version)" = "20.1.8" \
    && python3.11 --version \
    && cmake --version \
    && ninja --version \
    && gcc --version

CMD ["bash"]