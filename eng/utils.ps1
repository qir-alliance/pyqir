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


# Sets the LLVM path in the env section of the .cargo/config.toml
# Configures vscode rust analyzer to the correct features
function Use-LlvmInstallation {
    param (
        [string]$path
    )
    Write-BuildLog "Setting LLVM installation to: $path"

    $llvm_config_options = @(Get-Command (Join-Path $path "bin" "llvm-config*"))
    Assert ($llvm_config_options.Length -gt 0) "llvm config not found in $path"

    $llvm_config = $llvm_config_options[0].Source
    Write-BuildLog "Found llvm-config : $llvm_config"

    $version = [Version]::Parse("$(&$llvm_config --version)")
    $prefix = "LLVM_SYS_$($version.Major)1_PREFIX"

    Write-BuildLog "Setting $prefix set to: $path"

    if ($IsWindows) {
        # we have to escape '\'
        $path = $path.Replace('\', '\\')
    }

    # Create the workspace cofig.toml and set the LLVM_SYS env var
    New-Item -ItemType File -Path $CargoConfigToml -Force
    Add-Content -Path $CargoConfigToml -Value "[env]"
    Add-Content -Path $CargoConfigToml -Value "$($prefix) = `"$($path)`""

    # Add llvm feature version for rust-analyzer extension
    $vscode_settings = @{}
    if (!(Test-Path $VscodeSettingsJson)) {
        New-Item -ItemType File -Path $VscodeSettingsJson -Force
    }
    else {
        $vscode_settings = Get-Content $VscodeSettingsJson | ConvertFrom-Json -AsHashtable
    }

    $vscode_settings."rust-analyzer.cargo.features" = @("$(Get-LLVMFeatureVersion)")
    $vscode_settings | ConvertTo-Json | Set-Content -Path $VscodeSettingsJson
}

function Test-LlvmConfig {
    param (
        [string]$path
    )

    $llvm_config_options = @(Get-Command (Join-Path $path "bin" "llvm-config*"))
    if ($llvm_config_options.Length -eq 0) {
        return $false
    }
    $llvm_config = $llvm_config_options[0].Source
    try {
        exec {
            & $llvm_config --version | Out-Null
        }
    }
    catch {
        return $false
    }
    return $true
}

function Resolve-InstallationDirectory {
    $result = if (Test-Path env:\QIRLIB_LLVM_EXTERNAL_DIR) {
        $env:QIRLIB_LLVM_EXTERNAL_DIR
    }
    else {
        $packagePath = Get-DefaultInstallDirectory
        $packagePath
    }
    if (!(Test-Path $result)) {
        New-Item -ItemType Directory -Force $result | Out-Null
    }
    return $result
}

function Get-DefaultInstallDirectory {
    if (Test-Path env:\QIRLIB_CACHE_DIR) {
        $env:QIRLIB_CACHE_DIR
    }
    else {
        Join-Path $Target (Get-LLVMFeatureVersion)
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
    $found = (Test-Path env:\CONDA_PREFIX)
    if ($found) {
        $condaPrefix = $env:CONDA_PREFIX
        Write-BuildLog "Found conda environment: $condaPrefix"
    }
    $found
}

function Test-InVenvEnvironment {
    $found = (Test-Path env:\VIRTUAL_ENV)
    if ($found) {
        $venv = $env:VIRTUAL_ENV
        Write-BuildLog "Found venv environment: $venv"
    }
    $found
}

function Test-InVirtualEnvironment {
    (Test-InCondaEnvironment) -or (Test-InVenvEnvironment)
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
        id -u
    }
}

function Get-LinuxContainerGroupId {
    if (Test-Path env:\PYQIR_CONTAINER_GROUPID) {
        $env:PYQIR_CONTAINER_GROUPID
    }
    else {
        id -g
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

function Write-CacheStats {
    if (Test-CommandExists ccache) {
        Write-BuildLog "ccache config:"
        & { ccache --show-config } -ErrorAction SilentlyContinue
        Write-BuildLog "ccache stats:"
        & { ccache --show-stats } -ErrorAction SilentlyContinue
    }
    if (Test-CommandExists sccache) {
        Write-BuildLog "sccache config/stats:"
        & { sccache --show-stats } -ErrorAction SilentlyContinue
    }
}

function Get-LLVMFeatureVersion {
    if (Test-Path env:\PYQIR_LLVM_FEATURE_VERSION) {
        $env:PYQIR_LLVM_FEATURE_VERSION
    }
    else {
        # "llvm18-1", "llvm19-1", or "llvm20-1"
        "llvm20-1"
    }
}

function Get-CargoArgs {
    @(@("-vv", ""), @("--features", (Get-LLVMFeatureVersion)))
}

function Get-CliCargoArgs {
    Get-CargoArgs | ForEach-Object { $_ } | Where-Object { $_ -ne "" }
}

function Get-Wheels([string] $project) {
    $name = $project.Replace('-', '_')
    $pattern = Join-Path $Wheels $name-*.whl
    Get-Item -ErrorAction Ignore $pattern
}

function Get-Wheel([string] $project) {
    $wheels = @(Get-Wheels $project)
    Assert ($wheels.Length -gt 0) "Missing wheels for $project."
    Assert ($wheels.Length -le 1) "Multiple wheels for $project ($wheels). Clean the wheels directory."
    $wheels[0]
}

function Resolve-Python() {
    $hasPython = $null -ne (Get-Command python -ErrorAction Ignore)
    if ($hasPython -and ((python --version) -Match "Python 3.*")) {
        "python"
    }
    else {
        "python3"
    }
}

function Resolve-PythonRequirements([string[]] $projects) {
    $report = pip --quiet install --dry-run --ignore-installed --report - @projects | ConvertFrom-Json
    $report.install.metadata `
    | Where-Object { !$_.name.StartsWith("pyqir") } `
    | ForEach-Object { "$($_.name)==$($_.version)" }
}

function install-llvm {
    Param(
        [Parameter(Mandatory)]
        [string]$qirlibDir,
        [Parameter(Mandatory)]
        [ValidateSet("download", "build")]
        [string]$operation,
        [Parameter(Mandatory)]
        [ValidateSet("llvm18-1", "llvm19-1", "llvm20-1")]
        [string]$feature
    )

    $installationDirectory = Resolve-InstallationDirectory
    $clear_cache_var = $false
    if (!(Test-Path env:\QIRLIB_CACHE_DIR)) {
        $clear_cache_var = $true
        $env:QIRLIB_CACHE_DIR = $installationDirectory
    }
    try {
        Invoke-LoggedCommand -wd $qirlibDir {
            cargo build --release --no-default-features --features "$operation-llvm,$feature-no-llvm-linking" -vv
        }
    }
    finally {
        if ($clear_cache_var) {
            Remove-Item -Path Env:QIRLIB_CACHE_DIR
        }
    }
}

function Get-AuditWheelTag($python) {
    $arch = & $python -c "import platform; print(platform.machine())"
    if ($arch -eq "x86_64") {
        return "manylinux_2_35_x86_64"
    }
    elseif ($arch -eq "arm64" -or $arch -eq "aarch64") {
        return "manylinux_2_34_aarch64"
    }
    else {
        throw "Unsupported architecture $arch"
    }
}
