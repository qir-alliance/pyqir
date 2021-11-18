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

# Updates the cargo package version with the version specified.
function Restore-CargoTomlWithVersionInfo ($inputFile, $outputFile, $version) {
    $outFile = New-Item -ItemType File -Path $outputFile
    $inPackageSection = $false
    switch -regex -file $inputFile {
        "^\[(.+)\]" {
            # Section
            $section = $matches[1]
            $inPackageSection = $section -eq "package"
            Add-Content -Path $outFile -Value $_
        }
        "(.+?)\s*=(.*)" {
            # Key/Value
            $key, $value = $matches[1..2]
            if ($inPackageSection -and ($key -eq "version")) {
                $value = "version = ""$($version)"""
                Add-Content -Path $outFile -Value $value
            }
            else {
                Add-Content -Path $outFile -Value $_
            }
        }
        default {
            Add-Content -Path $outFile -Value $_
        }
    }
}

# Copies the default config.toml and sets the [env] config
# section to specify the variables needed for llvm-sys/inkwell
# This allows us to not need the user to specify env vars to build.
function Restore-ConfigTomlWithLlvmInfo {
    $cargoPath = Resolve-Path (Join-Path (Get-RepoRoot) '.cargo')
    $configTemplatePath = Join-Path $cargoPath config.toml.template
    $configPath = Join-Path $cargoPath config.toml

    # remove the old file if it exists.
    if (Test-Path $configPath) {
        Remove-Item $configPath
    }

    # ensure the output folder is there, `mkdir -p` equivalent
    New-Item -ItemType Directory -Path $cargoPath -Force | Out-Null

    # copy the template
    Copy-Item $configTemplatePath $configPath

    # append the env vars to the new config
    $installationDirectory = Resolve-InstallationDirectory
    Add-Content -Path $configPath -Value "[env]"
    Add-Content -Path $configPath -Value "LLVM_SYS_110_PREFIX = '$installationDirectory'"
}

function Get-LlvmSubmoduleSha {
    $status = Get-LlvmSubmoduleStatus
    $sha = $status.Substring(1, 9)
    $sha
}

function Get-LlvmSubmoduleStatus {
    Write-BuildLog "Detected submodules: $(git submodule status --cached)"
    $statusResult = exec -workingDirectory (Get-RepoRoot) { git submodule status --cached }
    # on all platforms, the status uses '/' in the module path.
    $status = $statusResult.Split([Environment]::NewLine) | Where-Object { $_.Contains("external/llvm-project") } | Select-Object -First 1
    $status
}

function Test-LlvmSubmoduleInitialized {
    $status = Get-LlvmSubmoduleStatus
    if ($status.Substring(0, 1) -eq "-") {
        Write-BuildLog "LLVM Submodule Uninitialized"
        return $false
    }
    else {
        Write-BuildLog "LLVM Submodule Initialized"
        return $true
    }
}

# Gets the LLVM package triple for the current platform
function Get-TargetTriple {
    $triple = "unknown"
    if ($IsWindows) {
        $triple = "x86_64-pc-windows-msvc-static"
    }
    elseif ($IsLinux) {
        $triple = "x86_64-unknown-linux-gnu"
    }
    elseif ($IsMacOS) {
        $triple = "x86_64-apple-darwin"
    }
    $triple
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

# Gets the LLVM version git hash
function Get-LlvmSha {
    $sha = exec { Get-LlvmSubmoduleSha }
    $sha
}

function Get-PackageName {
    $sha = Get-LlvmSha
    $TARGET_TRIPLE = Get-TargetTriple
    $packageName = "aq-llvm-$($TARGET_TRIPLE)-$($sha)"
    $packageName
}

function Get-DefaultInstallDirectory {
    if (Test-Path env:\PYQIR_CACHE_DIR) {
        $env:PYQIR_CACHE_DIR
    }
    else {
        Join-Path "$HOME" ".pyqir"
    }
}

function Get-AqCacheDirectory {
    $aqCacheDirectory = (Get-DefaultInstallDirectory)
    if (!(Test-Path $aqCacheDirectory)) {
        mkdir $aqCacheDirectory | Out-Null
    }
    Resolve-Path $aqCacheDirectory
}

function Get-InstallationDirectory {
    [CmdletBinding()]
    param (
        [Parameter()]
        [string]
        $packageName
    )
    $aqCacheDirectory = Get-AqCacheDirectory
    $packagePath = Join-Path $aqCacheDirectory $packageName
    $packagePath
}

function Resolve-InstallationDirectory {
    if (Test-Path env:\PYQIR_LLVM_EXTERNAL_DIR) {
        return $env:PYQIR_LLVM_EXTERNAL_DIR
    }
    else {
        $packageName = Get-PackageName

        $packagePath = Get-InstallationDirectory $packageName
        return $packagePath
    }
}
