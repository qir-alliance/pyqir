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

function Use-ExternalLlvmInstallation {
    Write-BuildLog "Using LLVM installation specified by QIRLIB_LLVM_EXTERNAL_DIR"
    Assert (Test-Path $env:QIRLIB_LLVM_EXTERNAL_DIR) "QIRLIB_LLVM_EXTERNAL_DIR folder does not exist"
    Use-LlvmInstallation $env:QIRLIB_LLVM_EXTERNAL_DIR
}

function Test-AllowedToDownloadLlvm {
    # If QIRLIB_DOWNLOAD_LLVM isn't set, we don't allow for download
    # If it is set, then we use its value
    ((Test-Path env:\QIRLIB_DOWNLOAD_LLVM) -and ($env:QIRLIB_DOWNLOAD_LLVM -eq $true))
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

function Get-ToxTarget {
    if (Test-MuslLinux) {
        "allmusl"
    }
    else {
        "all"
    }
}

function Test-MuslLinux {
    if ($IsLinux) {
        $triple = Get-LinuxTargetTriple
        $triple -eq "x86_64-unknown-linux-musl"
    }
    else {
        $false
    }
}

function Write-CacheStats {
    if (Test-CommandExists("ccache")) {
        Write-BuildLog "ccache config:"
        & { ccache --show-config } -ErrorAction SilentlyContinue
        Write-BuildLog "ccache stats:"
        & { ccache --show-stats } -ErrorAction SilentlyContinue
    }
    if (Test-CommandExists("sccache")) {
        Write-BuildLog "sccache config/stats:"
        & { sccache --show-stats } -ErrorAction SilentlyContinue
    }
}

function Build-PyQIR([string]$project) {
    $srcPath = $repo.root

    exec -workingDirectory (Join-Path $srcPath $project) {
        if (Test-InCondaEnvironment) {
            $build_extra_args = ""
            if (Test-MuslLinux) {
                $build_extra_args = "--skip-auditwheel"
            }
            Invoke-LoggedCommand {
                maturin build --release $build_extra_args --cargo-extra-args="-vv"
                maturin develop --release --cargo-extra-args="-vv"
                & $python -m pip install pytest
                & $python -m pytest
            }
        }
        else {
            Invoke-LoggedCommand {
                & $python -m pip install tox
            }
            Invoke-LoggedCommand {
                & $python -m tox -v -e (Get-ToxTarget)
            }
        }
    }
}

function Create-DocsEnv() {
    param(
        [string]
        $EnvironmentPath,
        [string]
        $RequirementsPath,
        [string[]]
        $ArtifactPaths
    )

    Write-Host "##[info]Creating virtual environment for use with docs at $EnvironmentPath..."
    python -m venv $EnvironmentPath

    $activateScript = (Join-Path $EnvironmentPath "bin" "Activate.ps1")
    if (-not (Test-Path $activateScript -ErrorAction SilentlyContinue)) {
        Get-ChildItem $EnvironmentPath | Write-Host
        throw "No activate script found for virtual environment at $EnvironmentPath; environment creation failed."
    }

    & $activateScript
    try {
        pip install -r $RequirementsPath
        foreach ($artifact in $ArtifactPaths) {
            pip install $artifact
        }
    }
    finally {
        deactivate
    }
}

function install-llvm {
    Param(
        [Parameter(Mandatory)]
        [string]$qirlibDir,
        [Parameter(Mandatory)]
        [ValidateSet("download", "build")]
        [string]$operation
    )

    $installationDirectory = Resolve-InstallationDirectory
    New-Item -ItemType Directory -Force $installationDirectory | Out-Null
    Use-LlvmInstallation $installationDirectory
    $clear_cache_var = $false
    if (!(Test-Path env:\QIRLIB_CACHE_DIR)) {
        $clear_cache_var = $true
        $env:QIRLIB_CACHE_DIR = $installationDirectory
    }
    try {
        Invoke-LoggedCommand -wd $qirlibDir {
            cargo build --release --no-default-features --features "$($operation)-llvm,no-llvm-linking" -vv
        }
    }
    finally {
        if ($clear_cache_var) {
            Remove-Item -Path Env:QIRLIB_CACHE_DIR
        }
    }   
}

function Get-CCacheParams {
    # only ccache is supported in the container for now.
    # we would need a way to specify which cache is used to
    # support both.
    if (Test-CommandExists("ccache")) {
        # we need to map the local cache dir into the
        # container. If the env var isn't set, ask ccache
        $cacheDir = ""
        if (Test-Path env:\CCACHE_DIR) {
            $cacheDir = $Env:CCACHE_DIR
        }
        else {
            $cacheDir = exec { ccache -k cache_dir }
        }
        if (![string]::IsNullOrWhiteSpace($cacheDir)) {
            New-Item -ItemType Directory -Force $cacheDir | Out-Null
            
            $cacheDir = Resolve-Path $cacheDir
            # mount the cache outside of any runner mappings
            $cacheMount = @("-v", "$($cacheDir):/ccache")
            $cacheEnv = @("-e", "CCACHE_DIR=`"/ccache`"")
            return $cacheMount, $cacheEnv
        }
    }
    return "", ""
}
