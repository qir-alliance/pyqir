# See here for image contents: https://github.com/microsoft/vscode-dev-containers/blob/v0.191.0/containers/codespaces-linux/.devcontainer/base.Dockerfile

FROM mcr.microsoft.com/vscode/devcontainers/universal:1-linux

USER root

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends ninja-build clang-11 clang-tidy-11 build-essential \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/

USER codespace
