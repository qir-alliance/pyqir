# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

FROM buildpack-deps:bionic-scm

WORKDIR /tmp

RUN wget https://packages.microsoft.com/config/ubuntu/18.04/packages-microsoft-prod.deb && \
    dpkg -i packages-microsoft-prod.deb && \
    rm packages-microsoft-prod.deb

RUN apt-get update && \
	apt-get install -y --no-install-recommends \
    apt-transport-https \
    powershell \
    python3-minimal \
    python3-pip \
    python3-setuptools \
    software-properties-common \
    sudo \
    && \
    python3 -m pip install -U pip && \
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