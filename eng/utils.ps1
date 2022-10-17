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
    $prefix = "LLVM_SYS_$($version.Major)0_PREFIX"

    Write-BuildLog "Setting $prefix set to: $path"

    if ($IsWindows) {
        # we have to escape '\'
        $path = $path.Replace('\', '\\')
    }

    # Create the workspace cofig.toml and set the LLVM_SYS env var
    New-Item -ItemType File -Path $repo.workspace_config_file -Force
    Add-Content -Path $repo.workspace_config_file -Value "[env]"
    Add-Content -Path $repo.workspace_config_file -Value "$($prefix) = `"$($path)`""

    # Add llvm feature version for rust-analyzer extension
    $vscode_settings = @{}
    if (!(Test-Path $repo.vscode_config_file)) {
        New-Item -ItemType File -Path $repo.vscode_config_file -Force
    }
    else {
        $vscode_settings = Get-Content $repo.vscode_config_file | ConvertFrom-Json -AsHashtable
    }

    $vscode_settings."rust-analyzer.cargo.features" = @("$(Get-LLVMFeatureVersion)")
    $vscode_settings | ConvertTo-Json | Set-Content -Path $repo.vscode_config_file
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
        Join-Path "$($repo.target)" "$(Get-LLVMFeatureVersion)"
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
    (Test-Path env:\CONDA_PREFIX)
}

function Test-InVenvEnvironment {
    (Test-Path env:\VIRTUAL_ENV)
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

function Get-LLVMFeatureVersion {
    if (Test-Path env:\PYQIR_LLVM_FEATURE_VERSION) {
        $env:PYQIR_LLVM_FEATURE_VERSION
    }
    else {
        # "llvm11-0", "llvm12-0", "llvm13-0", "llvm14-0"
        "llvm13-0"
    }
}

function Get-CargoArgs {
    @("-vv", "--features", (Get-LLVMFeatureVersion))
}

function Get-Wheel([string] $project) {
    $wheelProject = $project.Replace('-', '_')
    $pattern = Join-Path $repo.wheels "$wheelProject-*.whl"
    $wheels = @(Get-Item $pattern)
    if ($wheels.Length -eq 1) {
        $wheels[0]
    }
    elseif ($wheels.Length -gt 1) {
        throw "Multiple wheels matching $pattern. Clean the wheels directory."
    }
    else {
        throw "No wheels matching $pattern."
    }
}

function Resolve-PythonRequirements([string[]] $projects) {
    $report = pip --quiet install --dry-run --ignore-installed --report - @projects | ConvertFrom-Json
    $report.install.metadata `
    | Where-Object { !$_.name.StartsWith("pyqir") } `
    | ForEach-Object { "$($_.name)==$($_.version)" }
}

function Build-PyQIR([string] $project) {
    $projectDir = Join-Path $repo.root $project
    Invoke-LoggedCommand {
        $env:MATURIN_PEP517_ARGS = (Get-CargoArgs) -Join " "
        exec { pip --verbose wheel --wheel-dir $repo.wheels $projectDir }
        exec { pip install --force-reinstall "$(Get-Wheel $project)[test]" }
        exec -workingDirectory $projectDir { pytest }
    }
}

function Create-PyEnv() {
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
            pip install --force-reinstall $artifact
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
        [string]$operation,
        [Parameter(Mandatory)]
        [ValidateSet("llvm11-0", "llvm12-0", "llvm13-0", "llvm14-0")]
        [string]$feature
    )

    $installationDirectory = Resolve-InstallationDirectory
    New-Item -ItemType Directory -Force $installationDirectory | Out-Null
    $clear_cache_var = $false
    if (!(Test-Path env:\QIRLIB_CACHE_DIR)) {
        $clear_cache_var = $true
        $env:QIRLIB_CACHE_DIR = $installationDirectory
    }
    try {
        Invoke-LoggedCommand -wd $qirlibDir {
            cargo build --release --no-default-features --features "$($operation)-llvm,$feature-no-llvm-linking" -vv
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
