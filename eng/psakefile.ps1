# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

properties {
    $repo = @{}
    $repo.root = Resolve-Path (Split-Path -parent $PSScriptRoot)

    $pyqir = @{}

    $pyqir.parser = @{}
    $pyqir.parser.name = "pyqir-parser"
    $pyqir.parser.dir = Join-Path $repo.root "pyqir-parser"

    $pyqir.generator = @{}
    $pyqir.generator.name = "pyqir-generator"
    $pyqir.generator.dir = Join-Path $repo.root "pyqir-generator"
    $pyqir.generator.examples_dir = Join-Path $repo.root "examples" "generator"

    $pyqir.jit = @{}
    $pyqir.jit.name = "pyqir-jit"
    $pyqir.jit.dir = Join-Path $repo.root "pyqir-jit"
    $pyqir.jit.examples_dir = Join-Path $repo.root "examples" "jit"

    $docs = @{}
    $docs.root = Join-Path $repo.root "docs"
    $docs.build = @{}
    $docs.build.dir = Join-Path $docs.root "_build"
    $docs.build.opts = @()

    $wheelhouse = Join-Path $repo.root "target" "wheels" "*.whl"
}

Include settings.ps1
Include utils.ps1

Task default -Depends checks, parser, generator, jit, run-examples, run-examples-in-containers

Task checks -Depends cargo-fmt, cargo-clippy, pyqir-tests

Task cargo-fmt {
    Invoke-LoggedCommand -workingDirectory $repo.root -errorMessage "Please run 'cargo fmt --all' before pushing" {
        cargo fmt --all -- --check
    }
}

Task cargo-clippy -Depends init {
    Invoke-LoggedCommand -workingDirectory $repo.root -errorMessage "Please fix the above clippy errors" {
        $extraArgs = (Test-CI) ? @("--", "-D", "warnings") : @()
        cargo clippy --workspace --all-targets --all-features @extraArgs
    }
}

Task init {
    Restore-ConfigTomlWithLlvmInfo
    Test-Prerequisites
    Initialize-Environment
}

Task generator -Depends init {
    Build-PyQIR($pyqir.generator.name)
}

Task jit -Depends init {
    Build-PyQIR($pyqir.jit.name)
}

Task parser -Depends init {
    Build-PyQIR($pyqir.parser.name)
}

Task pyqir-tests -Depends init {
    Invoke-LoggedCommand -workingDirectory (Join-Path $repo.root pyqir-tests) {
        python -m tox -e test
    }
}

Task rebuild -Depends generator, jit, parser
Task wheelhouse `
    -Precondition { -not (Test-Path $wheelhouse -ErrorAction SilentlyContinue) } `
{ Invoke-Task rebuild }

Task docs -Depends wheelhouse {
    # - Install artifacts into new venv along with sphinx.
    # - Run sphinx from within new venv.
    $envPath = Join-Path $repo.root ".docs-venv"
    $sphinxOpts = $docs.build.opts
    Create-DocsEnv `
        -EnvironmentPath $envPath `
        -RequirementsPath (Join-Path $repo.root "eng" "docs-requirements.txt") `
        -ArtifactPaths (Get-Item $wheelhouse)
    & (Join-Path $envPath "bin" "Activate.ps1")
    try {
        sphinx-build -M html $docs.root $docs.build.dir @sphinxOpts
    }
    finally {
        deactivate
    }
}

function Use-ExternalLlvmInstallation {
    Write-BuildLog "Using LLVM installation specified by PYQIR_LLVM_EXTERNAL_DIR"
    Assert (Test-Path $env:PYQIR_LLVM_EXTERNAL_DIR) "PYQIR_LLVM_EXTERNAL_DIR folder does not exist"
    Use-LlvmInstallation $env:PYQIR_LLVM_EXTERNAL_DIR
}

function Test-AllowedToDownloadLlvm {
    # If PYQIR_DOWNLOAD_LLVM isn't set, we allow for download
    # If it is set, then we use its value
    !((Test-Path env:\PYQIR_DOWNLOAD_LLVM) -and ($env:PYQIR_DOWNLOAD_LLVM -eq $false))
}

function Get-LlvmDownloadBaseUrl {
    if (Test-Path env:\PYQIR_LLVM_BUILDS_URL) {
        $env:PYQIR_LLVM_BUILDS_URL
    }
    else
    { "https://msquantumpublic.blob.core.windows.net/llvm-builds" }
}

function Get-PackageExt {
    $extension = ".tar.gz"
    if ($IsWindows) {
        $extension = ".zip"
    }
    $extension
}

function Get-LlvmArchiveUrl {
    $extension = Get-PackageExt
    $baseUrl = Get-LlvmDownloadBaseUrl
    "$baseUrl/$($packageName)$extension"
}

function Get-LlvmArchiveShaUrl {
    $extension = Get-PackageExt
    $baseUrl = Get-LlvmDownloadBaseUrl
    "$baseUrl/$($packageName)$extension.sha256"
}

function Get-LlvmArchiveFileName {
    $packageName = Get-PackageName
    $extension = Get-PackageExt
    "$($packageName)$extension"
}

function Get-LlvmArchiveShaFileName {
    $filename = Get-LlvmArchiveFileName
    "$filename.sha256"
}

function Install-LlvmFromBuildArtifacts {
    [CmdletBinding()]
    param (
        [Parameter()]
        [string]
        $packagePath
    )

    $outFile = Join-Path $($env:TEMP) (Get-LlvmArchiveFileName)
    if ((Test-Path $outFile)) {
        Remove-Item $outFile
    }

    $archiveUrl = Get-LlvmArchiveUrl
    Write-BuildLog "Dowloading $archiveUrl to $outFile"
    Invoke-WebRequest -Uri $archiveUrl -OutFile $outFile

    $shaFile = Join-Path $($env:TEMP) (Get-LlvmArchiveShaFileName)
    if ((Test-Path $shaFile)) {
        Remove-Item $shaFile
    }

    $sha256Url = Get-LlvmArchiveShaUrl
    Write-BuildLog "Dowloading $sha256Url to $shaFile"
    Invoke-WebRequest -Uri $sha256Url -OutFile $shaFile
    Write-BuildLog "Calculating hash for $outFile"
    $calculatedHash = (Get-FileHash -Path $outFile -Algorithm SHA256).Hash

    Write-BuildLog "Reading hash from $shaFile"
    $expectedHash = (Get-Content -Path $shaFile)

    Assert ("$calculatedHash" -eq "$expectedHash") "The calculated hash $calculatedHash did not match the expected hash $expectedHash"

    $packagesRoot = Get-AqCacheDirectory
    if ($IsWindows) {
        Expand-Archive -Path $outFile -DestinationPath $packagesRoot
    }
    else {
        tar -zxvf $outFile -C $packagesRoot
    }

    $packageName = Get-PackageName
    $packagePath = Get-InstallationDirectory $packageName
    Use-LlvmInstallation $packagePath
}

function Install-LlvmFromSource {
    [CmdletBinding()]
    param (
        [Parameter()]
        [string]
        $packagePath
    )
    $Env:PKG_NAME = Get-PackageName
    $Env:CMAKE_INSTALL_PREFIX = $packagePath
    $Env:INSTALL_LLVM_PACKAGE = $true
    Assert $false -failureMessage "TODO: Migration in progress"
    . (Join-Path (Get-RepoRoot) "build" "llvm.ps1")
    Use-LlvmInstallation $packagePath
}

function Test-Prerequisites {
    if (!(Test-LlvmSubmoduleInitialized)) {
        Write-BuildLog "llvm-project submodule isn't initialized"
        Write-BuildLog "Initializing submodules: git submodule init"
        exec -workingDirectory ($repo.root ) { git submodule init }
        Write-BuildLog "Updating submodules: git submodule update --depth 1 --recursive"
        exec -workingDirectory ($repo.root ) { git submodule update --depth 1 --recursive }
    }
    Assert (Test-LlvmSubmoduleInitialized) "Failed to read initialized llvm-project submodule"
}

function Initialize-Environment {
    # if an external LLVM is specified, make sure it exist and
    # skip further bootstapping
    if (Test-Path env:\PYQIR_LLVM_EXTERNAL_DIR) {
        Use-ExternalLlvmInstallation
    }
    else {
        $llvmSha = Get-LlvmSha
        Write-BuildLog "llvm-project sha: $llvmSha"
        $packageName = Get-PackageName

        $packagePath = Get-InstallationDirectory $packageName
        if (Test-Path $packagePath) {
            Write-BuildLog "LLVM target $($llvmSha) is already installed."
            # LLVM is already downloaded
            Use-LlvmInstallation $packagePath
        }
        else {
            Write-BuildLog "LLVM target $($llvmSha) is not installed."
            if (Test-AllowedToDownloadLlvm) {
                Write-BuildLog "Downloading LLVM target $packageName "
                Install-LlvmFromBuildArtifacts $packagePath
            }
            else {
                Write-BuildLog "Downloading LLVM Disabled, building from source."
                # We don't have an external LLVM installation specified
                # We are not downloading LLVM
                # So we need to build it.
                Install-LlvmFromSource $packagePath
            }
        }
    }
}


# Only run the nested ManyLinux container
# build on Linux while not in a dev container
function Test-RunInContainer {
    if ($IsLinux -and (Test-CI)) {
        # If we are in a dev container, our workspace is already
        # mounted into the container. If we try to mount our 'local' workspace
        # into a nested container it will silently fail to mount.
        !(Test-InDevContainer)
    }
    else {
        $false
    }
}

function Build-PyQIR([string]$project) {
    $srcPath = $repo.root
    $installationDirectory = Resolve-InstallationDirectory

    if (Test-RunInContainer) {
        function Build-ContainerImage {
            Write-BuildLog "Building container image manylinux-llvm-builder"
            Invoke-LoggedCommand -workingDirectory (Join-Path $srcPath eng) {
                Get-Content manylinux.Dockerfile | docker build -t manylinux2014_x86_64_maturin -
            }
        }
        function Invoke-ContainerImage {
            Write-BuildLog "Running container image:"
            $ioVolume = "$($srcPath):/io"
            $llvmVolume = "$($installationDirectory):/usr/lib/llvm"
            $userSpec = ""

            Invoke-LoggedCommand {
                docker run --rm $userSpec -v $ioVolume -v $llvmVolume -e LLVM_SYS_110_PREFIX=/usr/lib/llvm -w /io/$project manylinux2014_x86_64_maturin conda run --no-capture-output cargo test --release --lib -vv -- --nocapture
            }

            Invoke-LoggedCommand {
                docker run --rm $userSpec -v $ioVolume -v $llvmVolume -e LLVM_SYS_110_PREFIX=/usr/lib/llvm -w /io/$project manylinux2014_x86_64_maturin conda run --no-capture-output /usr/bin/maturin build --release
            }

            Invoke-LoggedCommand {
                docker run --rm $userSpec -v $ioVolume -v $llvmVolume -e LLVM_SYS_110_PREFIX=/usr/lib/llvm -w /io/$project manylinux2014_x86_64_maturin conda run --no-capture-output python -m tox -e test
            }
            
            Invoke-LoggedCommand {
                docker run --rm $userSpec -v $ioVolume -v $llvmVolume -e LLVM_SYS_110_PREFIX=/usr/lib/llvm -w /io/$project manylinux2014_x86_64_maturin conda run --no-capture-output cargo test --release -vv -- --nocapture
            }
        }

        Build-ContainerImage
        Invoke-ContainerImage
    }
    else {

        exec -workingDirectory (Join-Path $srcPath $project) {
            Invoke-LoggedCommand {
                & $python -m pip install tox
            }

            Invoke-LoggedCommand {
                & $python -m tox -e test
            }

            Invoke-LoggedCommand {
                & $python -m tox -e pack
            }
        }

        Invoke-LoggedCommand -workingDirectory $srcPath {
            & cargo test --release -vv -- --nocapture
        }
    }
}

task run-examples-in-containers -precondition { $IsLinux -and (Test-CI) } {
    $userName = [Environment]::UserName
    $userId = $(id -u)
    $groupId = $(id -g)
    $images = @("buster", "bullseye", "bionic", "focal")
    foreach ($image in $images) {
        exec -workingDirectory (Join-Path $repo.root "eng") {
            get-content "$($image).Dockerfile" | docker build --build-arg USERNAME=$userName --build-arg USER_UID=$userId --build-arg USER_GID=$groupId -t "$image-samples" -
        }
        exec {
            docker run --rm --user $userName -v "$($repo.root):/home/$userName" "$image-samples" build.ps1 -t run-examples
        }
    }
}

task run-examples {   
    exec -workingDirectory $repo.root {
        $wheels = Join-Path $repo.root "target" "wheels"
        & $python -m pip install -r requirements.txt --no-index --find-links=$wheels -v
    }

    exec -workingDirectory $pyqir.generator.examples_dir {
        & $python -m pip install -r requirements.txt
        & $python "bell_pair.py" | Tee-Object -Variable bell_pair_output
        $bell_first_line = $($bell_pair_output | Select-Object -first 1)
        $bell_expected = "; ModuleID = 'Bell'"
        Assert ($bell_first_line -eq $bell_expected) "Expected $bell_expected found $bell_first_line"

        $bz_output = (Join-Path $($env:TEMP) "bz.ll")
        & $python "mock_to_qir.py" -o $bz_output "bernstein_vazirani.txt" 7
        $bz_first_line = Get-Content $bz_output | Select-Object -first 1
        $bz_expected = "; ModuleID = 'bernstein_vazirani'"
        Assert ($bz_first_line -eq $bz_expected) "Expected $bz_expected found $bz_first_line"
    }

    exec -workingDirectory $pyqir.jit.examples_dir {
        & $python "bernstein_vazirani.py" | Tee-Object -Variable bz_output
        $bz_first_lines = @($bz_output | Select-Object -first 5)
        $bz_expected = @(
            "# output from GateLogger",
            "qubits[6]",
            "out[6]",
            "x qubit[5]",
            "h qubit[0]"
        )
        Assert (@(Compare-Object $bz_first_lines $bz_expected).Length -eq 0) "Expected $bz_expected found $bz_first_lines"
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
