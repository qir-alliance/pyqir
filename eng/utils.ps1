# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

if (!(Test-Path function:\Get-RepoRoot)) {
    # pin working directory to this repo in case
    # we are ever in a submodule
    function Get-RepoRoot {
        exec -workingDirectory $PSScriptRoot {
            git rev-parse --show-toplevel
        }
    }
}

# Fix temp path for non-windows platforms if missing
if (!(Test-Path env:\TEMP)) {
    $env:TEMP = [System.IO.Path]::GetTempPath()
}

####
# Utilities
####

# returns true if the script is running on a build agent, false otherwise
function Test-CI {
    if (Test-Path env:\TF_BUILD) {
        $true
    }
    elseif ((Test-Path env:\CI)) {
        $env:CI -eq $true
    }
    else {
        $false
    }
}

# Writes an Azure DevOps message with default debug severity
function Write-BuildLog {
    param (
        [Parameter(Mandatory = $true)]
        [string]$message,
        [Parameter(Mandatory = $false)]
        [ValidateSet("group", "warning", "error", "section", "debug", "command", "endgroup")]
        [string]$severity = "debug"
    )
    Write-Host "##[$severity]$message"
}

# Returns true if a command with the specified name exists.
function Test-CommandExists($name) {
    $null -ne (Get-Command $name -ErrorAction SilentlyContinue)
}

# Returns true if the current environment is a dev container.
function Test-InDevContainer {
    $IsLinux -and (Test-Path env:\IN_DEV_CONTAINER)
}

# This method should be able to be removed when Rust 1.56 is released
# which contains the feature for env sections in the .cargo/config.toml
function Use-LlvmInstallation {
    param (
        [string]$path
    )
    Write-BuildLog "LLVM installation set to: $path"
    $env:LLVM_SYS_110_PREFIX = $path
}

function Resolve-InstallationDirectory {
    if (Test-Path env:\QIRLIB_LLVM_EXTERNAL_DIR) {
        return $env:QIRLIB_LLVM_EXTERNAL_DIR
    }
    else {
        $packagePath = Get-DefaultInstallDirectory
        return $packagePath
    }
}

function Get-DefaultInstallDirectory {
    if (Test-Path env:\QIRLIB_CACHE_DIR) {
        $env:QIRLIB_CACHE_DIR
    }
    else {
        Join-Path "$HOME" ".pyqir"
    }
}

# Executes the supplied script block using psake's exec
# Warning: Do not use this command on anything that contains
#          sensitive information!
function Invoke-LoggedCommand {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [scriptblock]$cmd,

        [string]$errorMessage = $null,

        [int]$maxRetries = 0,

        [string]$retryTriggerErrorPattern = $null,

        [Alias("wd")]
        [string]$workingDirectory = $null
    )

    Write-BuildLog "Invoke-LoggedCommand in $workingDirectory`:"
    Write-BuildLog $ExecutionContext.InvokeCommand.ExpandString($cmd).Trim() "command"

    # errorMessage pulls default values from psake. We
    # only want to pass the param if we want to override.
    # all other parameters have safe defaults.
    $extraArgs = $errorMessage ? @{ "errorMessage" = $errorMessage } : @{};
    exec $cmd @extraArgs `
        -maxRetries $maxRetries `
        -retryTriggerErrorPattern $retryTriggerErrorPattern `
        -workingDirectory $workingDirectory
}

function Test-InCondaEnvironment {
    (Test-Path env:\CONDA_PREFIX) -or (Test-Path env:\CONDA_ROOT)
}

function Get-LinuxTargetTriple {
    $triple = rustc -vV | sed -n 's|host: ||p'
    $triple
}

function Get-LinuxContainerUserId {
    if (Test-Path env:\PYQIR_CONTAINER_USERID) {
        $env:PYQIR_CONTAINER_USERID
    }
    else {
        "$(id -u)"
    }
}

function Get-LinuxContainerGroupId {
    if (Test-Path env:\PYQIR_CONTAINER_GROUPID) {
        $env:PYQIR_CONTAINER_GROUPID
    }
    else {
        "$(id -g)"
    }
}

function Get-LinuxContainerUserName {
    if (Test-Path env:\PYQIR_CONTAINER_USERNAME) {
        $env:PYQIR_CONTAINER_USERNAME
    }
    else {
        [Environment]::UserName
    }
}
