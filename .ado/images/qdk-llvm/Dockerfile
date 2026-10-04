ARG BASE_REGISTRY=qdkacr-cfgqhwddajdaebhg.azurecr.io
ARG LLVM_INSTALL_PREFIX=/opt/llvm-20

FROM ${BASE_REGISTRY}/qdk-image:latest AS builder

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

FROM ${BASE_REGISTRY}/qdk-image:latest

ARG LLVM_INSTALL_PREFIX

COPY --from=builder ${LLVM_INSTALL_PREFIX}/ ${LLVM_INSTALL_PREFIX}/

ENV PATH="${LLVM_INSTALL_PREFIX}/bin:${PATH}"
ENV LLVM_SYS_201_PREFIX="${LLVM_INSTALL_PREFIX}"

WORKDIR /work

RUN llvm-config --version

RUN test "$(llvm-config --prefix)" = "${LLVM_INSTALL_PREFIX}" \
    && echo "LLVM host target: $(llvm-config --host-target)" \
    && echo "LLVM targets built: $(llvm-config --targets-built)" \
    && echo "LLVM build mode: $(llvm-config --build-mode)" \
    && echo "LLVM assertions: $(llvm-config --assertion-mode)" \
    && echo "LLVM RTTI: $(llvm-config --has-rtti)" \
    && echo "LLVM shared mode: $(llvm-config --shared-mode)" \
    && echo "LLVM library directory: $(llvm-config --libdir)" \
    && echo "LLVM version: $(llvm-config --version)"
