#!/usr/bin/env pwsh

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "& '$(Join-Path $pwd eng build.ps1)' $args"
exit $LASTEXITCODE
