# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

properties {
    $repo = @{}
    $repo.root = Resolve-Path (Split-Path -parent $PSScriptRoot)

    $pyqir = @{}

    $pyqir.qirlib = @{}
    $pyqir.qirlib.name = "qirlib"
    $pyqir.qirlib.dir = Join-Path $repo.root "qirlib"

    $pyqir.parser = @{}
    $pyqir.parser.name = "pyqir-parser"
    $pyqir.parser.dir = Join-Path $repo.root "pyqir-parser"

    $pyqir.generator = @{}
    $pyqir.generator.name = "pyqir-generator"
    $pyqir.generator.dir = Join-Path $repo.root "pyqir-generator"
    $pyqir.generator.examples_dir = Join-Path $repo.root "examples" "generator"

    $pyqir.evaluator = @{}
    $pyqir.evaluator.name = "pyqir-evaluator"
    $pyqir.evaluator.dir = Join-Path $repo.root "pyqir-evaluator"
    $pyqir.evaluator.examples_dir = Join-Path $repo.root "examples" "evaluator"

    $docs = @{}
    $docs.root = Join-Path $repo.root "docs"
    $docs.build = @{}
    $docs.build.dir = Join-Path $docs.root "_build"
    $docs.build.opts = @()

    $wheelhouse = Join-Path $repo.root "target" "wheels" "*.whl"
}

Include settings.ps1
Include utils.ps1

Task default -Depends checks, pyqir-tests, parser, generator, evaluator, run-examples, run-examples-in-containers

Task checks -Depends cargo-fmt, cargo-clippy

Task rebuild -Depends generator, evaluator, parser

Task cargo-fmt {
    Invoke-LoggedCommand -workingDirectory $repo.root -errorMessage "Please run 'cargo fmt --all' before pushing" {
        cargo fmt --all -- --check
    }
}

Task cargo-clippy -Depends init {
    Invoke-LoggedCommand -workingDirectory $repo.root -errorMessage "Please fix the above clippy errors" {
        $extraArgs = (Test-CI) ? @("--", "-D", "warnings") : @()
        cargo clippy --workspace --all-targets @extraArgs
    }
}

Task generator -Depends init {
    Build-PyQIR($pyqir.generator.name)
}

Task evaluator -Depends init {
    Build-PyQIR($pyqir.evaluator.name)
}

Task parser -Depends init {
    Build-PyQIR($pyqir.parser.name)
}

Task pyqir-tests -Depends init {
    Invoke-LoggedCommand -workingDirectory (Join-Path $repo.root pyqir-tests) {
        & $python -m pip install tox
        & $python -m tox -e test
    }
}

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
    Write-BuildLog "Using LLVM installation specified by QIRLIB_LLVM_EXTERNAL_DIR"
    Assert (Test-Path $env:QIRLIB_LLVM_EXTERNAL_DIR) "QIRLIB_LLVM_EXTERNAL_DIR folder does not exist"
    Use-LlvmInstallation $env:QIRLIB_LLVM_EXTERNAL_DIR
}

function Test-AllowedToDownloadLlvm {
    # If QIRLIB_DOWNLOAD_LLVM isn't set, we allow for download
    # If it is set, then we use its value
    !((Test-Path env:\QIRLIB_DOWNLOAD_LLVM) -and ($env:QIRLIB_DOWNLOAD_LLVM -eq $false))
}

task init {
    # if an external LLVM is specified, make sure it exist and
    # skip further bootstapping
    if (Test-Path env:\QIRLIB_LLVM_EXTERNAL_DIR) {
        Use-ExternalLlvmInstallation
    }
    else {
        $packagePath = Resolve-InstallationDirectory
        if (Test-Path $packagePath) {
            Write-BuildLog "LLVM target is already installed."
            # LLVM is already downloaded
            Use-LlvmInstallation $packagePath
        }
        else {
            Write-BuildLog "LLVM target is not installed."
            if (Test-AllowedToDownloadLlvm) {
                Write-BuildLog "Downloading LLVM target"
                Invoke-Task "install-llvm-from-archive"
            }
            else {
                Write-BuildLog "Downloading LLVM Disabled, building from source."
                # We don't have an external LLVM installation specified
                # We are not downloading LLVM
                # So we need to build it.
                Invoke-Task "install-llvm-from-source"
            }
        }
    }
}

task install-llvm-from-archive {
    $cache = Resolve-InstallationDirectory
    Use-LlvmInstallation $cache
    $clear_cache_var = $false
    if (!(Test-Path env:\QIRLIB_CACHE_DIR)) {
        $clear_cache_var = $true
        $env:QIRLIB_CACHE_DIR = $cache
    }
    try {
        Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
            cargo build --release --features install-llvm -vv
        }
    }
    finally {
        if ($clear_cache_var) {
            Remove-Item -Path Env:QIRLIB_CACHE_DIR
        }
    }
}

task install-llvm-from-source {
    $cache = Resolve-InstallationDirectory
    Use-LlvmInstallation $cache
    if ($IsLinux) {
        Build-ContainerImage $repo.root
        $srcPath = $repo.root
        $ioVolume = "$($srcPath):/io"
        $install_volume = "$($cache):$($cache)"
        Invoke-LoggedCommand {
            docker run --rm $userSpec -v $ioVolume -v $install_volume -w /io/qirlib manylinux2014_x86_64_maturin conda run --no-capture-output cargo build --release --features build-llvm -vv
        }
    }
    else {
        if ($IsWindows) {
            Include vcvars.ps1
        }
        $clear_cache_var = $false
        if (!(Test-Path env:\QIRLIB_CACHE_DIR)) {
            $clear_cache_var = $true
            $env:QIRLIB_CACHE_DIR = $cache
        }
        try {
            Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
                cargo build --release --features build-llvm -vv
            }
        }
        finally {
            if ($clear_cache_var) {
                Remove-Item -Path Env:QIRLIB_CACHE_DIR
            }
        }
    }
}

task package-llvm {
    if (Test-RunInContainer) {
        Build-ContainerImage $repo.root
        $srcPath = $repo.root
        $ioVolume = "$($srcPath):/io"
        Invoke-LoggedCommand {
            docker run --rm $userSpec -v $ioVolume -w /io/qirlib -e QIRLIB_PKG_DEST=/io/target manylinux2014_x86_64_maturin conda run --no-capture-output cargo build --release --features package-llvm -vv
        }
    }
    else {
        if ($IsWindows) {
            Include vcvars.ps1
        }
        $clear_pkg_dest_var = $false
        if (!(Test-Path env:\QIRLIB_PKG_DEST)) {
            $clear_pkg_dest_var = $true
            $env:QIRLIB_PKG_DEST = Join-Path $repo.root "target"
        }
        try {
            Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
                cargo build --release --features package-llvm -vv
            }
        }
        finally {
            if ($clear_pkg_dest_var) {
                Remove-Item -Path Env:QIRLIB_PKG_DEST
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

function Build-ContainerImage([string]$srcPath) {
    Write-BuildLog "Building container image manylinux-llvm-builder"
    Invoke-LoggedCommand -workingDirectory (Join-Path $srcPath eng) {
        Get-Content manylinux.Dockerfile | docker build -t manylinux2014_x86_64_maturin -
    }
}

function Build-PyQIR([string]$project) {
    $srcPath = $repo.root
    $installationDirectory = Resolve-InstallationDirectory

    if (Test-RunInContainer) {
        Build-ContainerImage $srcPath
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

task run-examples-in-containers -precondition { Test-RunInContainer } {
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

    exec -workingDirectory $pyqir.evaluator.examples_dir {
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
