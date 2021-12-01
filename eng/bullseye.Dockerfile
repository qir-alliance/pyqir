# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

FROM buildpack-deps:bullseye-scm

WORKDIR /tmp

RUN apt-get update && \
	apt-get install -y --no-install-recommends \
    apt-transport-https \
    python3-minimal \
    python3-pip \
    sudo \
    && \
    python3 -m pip install -U pip && \
    rm -rf /var/lib/apt/lists/*

RUN wget https://github.com/PowerShell/PowerShell/releases/download/v7.2.0/powershell-lts_7.2.0-1.deb_amd64.deb && \
    apt-get update && \
    apt-get install -y ./powershell-lts_7.2.0-1.deb_amd64.deb && \
    rm powershell-lts_7.2.0-1.deb_amd64.deb && \
    rm -rf /var/lib/apt/lists/*

ARG USERNAME=pyqir
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME
RUN useradd -s /bin/bash --uid $USER_UID --gid $USERNAME -m $USERNAME
RUN echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME
RUN chmod 0440 /etc/sudoers.d/$USERNAME

USER ${USER_UID}:${USER_GID}

WORKDIR /home/${USERNAME}

ENTRYPOINT [ "pwsh" ]