# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

properties {
    $repo = @{}
    $repo.root = Resolve-Path (Split-Path -parent $PSScriptRoot)
    $repo.target = Join-Path $repo.root "target"
    $repo.dot_cargo = Join-Path $repo.root ".cargo"
    $repo.workspace_config_file = Join-Path $repo.dot_cargo "config.toml"
    $repo.dot_vscode = Join-Path $repo.root ".vscode"
    $repo.vscode_config_file = Join-Path $repo.dot_vscode "settings.json"

    $pyqir = @{}

    $pyqir.qirlib = @{}
    $pyqir.qirlib.name = "qirlib"
    $pyqir.qirlib.dir = Join-Path $repo.root $pyqir.qirlib.name

    $pyqir.meta = @{}
    $pyqir.meta.name = "pyqir"
    $pyqir.meta.dir = Join-Path $repo.root $pyqir.meta.name

    $pyqir.parser = @{}
    $pyqir.parser.name = "pyqir-parser"
    $pyqir.parser.dir = Join-Path $repo.root $pyqir.parser.name
    $pyqir.parser.python_dir = Join-Path $pyqir.parser.dir "pyqir" "parser"

    $pyqir.generator = @{}
    $pyqir.generator.name = "pyqir-generator"
    $pyqir.generator.dir = Join-Path $repo.root $pyqir.generator.name
    $pyqir.generator.examples_dir = Join-Path $repo.root "examples" "generator"
    $pyqir.generator.python_dir = Join-Path $pyqir.generator.dir "pyqir" "generator"

    $pyqir.evaluator = @{}
    $pyqir.evaluator.name = "pyqir-evaluator"
    $pyqir.evaluator.dir = Join-Path $repo.root $pyqir.evaluator.name
    $pyqir.evaluator.examples_dir = Join-Path $repo.root "examples" "evaluator"
    $pyqir.evaluator.python_dir = Join-Path $pyqir.evaluator.dir "pyqir" "evaluator"

    $pyqir.tests = @{}
    $pyqir.tests.name = "pyqir-tests"
    $pyqir.tests.dir = Join-Path $repo.root $pyqir.tests.name

    $docs = @{}
    $docs.root = Join-Path $repo.root "docs"
    $docs.build = @{}
    $docs.build.dir = Join-Path $docs.root "_build"
    $docs.build.opts = @()

    $rust = @{}
    $rust.version = "1.57.0"

    $linux = @{}
    $linux.manylinux_tag = "manylinux2014_x86_64_maturin"
    $linux.manylinux_root = "/io"
    $linux.musllinux_tag = "musllinux_1_2_x86_64_maturin"
    $linux.musllinux_root = "/oi"

    [Diagnostics.CodeAnalysis.SuppressMessage("PSUseDeclaredVarsMoreThanAssignments", "")]
    $wheelhouse = Join-Path $repo.root "target" "wheels" "*.whl"
}

include settings.ps1
include utils.ps1

task default -depends qirlib, pyqir-tests, parser, generator, evaluator, metawheel, run-examples

task manylinux -depends build-manylinux-container-image, run-manylinux-container-image, run-examples-in-containers 

task musllinux -depends build-musllinux-container-image, run-musllinux-container-image, run-examples-in-musl-containers

task checks -depends cargo-fmt, cargo-clippy, mypy

task build -depends qirlib, generator, evaluator, parser

task run-manylinux-container-image -preaction { Write-CacheStats } -postaction { Write-CacheStats } {
    $srcPath = $repo.root

    # For any of the volumes mapped, if the dir doesn't exist,
    # docker will create it and it will be owned by root and
    # the caching/install breaks with permission errors.
    # New-Item is idempotent so we don't need to check for existence

    $cacheMount, $cacheEnv = Get-CCacheParams

    Write-BuildLog "Running container image: $($linux.manylinux_tag)"
    $ioVolume = "$($srcPath):$($linux.manylinux_root)"
    $userName = Get-LinuxContainerUserName

    Invoke-LoggedCommand {
        docker run --rm --user $userName -v $ioVolume @cacheMount @cacheEnv -e QIRLIB_CACHE_DIR="/tmp/llvm" -w "$($linux.manylinux_root)" "$($linux.manylinux_tag)" conda run --no-capture-output pwsh build.ps1 -t default
    }
}

task run-musllinux-container-image -preaction { Write-CacheStats } -postaction { Write-CacheStats } {
    $srcPath = $repo.root

    # For any of the volumes mapped, if the dir doesn't exist,
    # docker will create it and it will be owned by root and
    # the caching/install breaks with permission errors.
    # New-Item is idempotent so we don't need to check for existence

    $cacheMount, $cacheEnv = Get-CCacheParams

    Write-BuildLog "Running container image: $($linux.musllinux_tag)"
    $ioVolume = "$($srcPath):$($linux.musllinux_root)"
    $userName = Get-LinuxContainerUserName
    if (Test-CI) {
        $userName = "root"
    }
    Invoke-LoggedCommand {
        docker run --rm --user $userName -v $ioVolume @cacheMount @cacheEnv -e QIRLIB_CACHE_DIR="/tmp/llvm" -w "$($linux.musllinux_root)" "$($linux.musllinux_tag)" pwsh build.ps1
    }
}

task cargo-fmt {
    Invoke-LoggedCommand -workingDirectory $repo.root -errorMessage "Please run 'cargo fmt --all' before pushing" {
        cargo fmt --all -- --check
    }
}

task cargo-clippy -depends init {
    Invoke-LoggedCommand -workingDirectory $repo.root -errorMessage "Please fix the above clippy errors" {
        cargo clippy --workspace --all-targets @("$($env:CARGO_EXTRA_ARGS)" -split " ") -- -D warnings
    }
}

task mypy -depends python-requirements {
    exec {
        pip install mypy
    }

    Invoke-LoggedCommand -workingDirectory $repo.root -errorMessage "Please fix the above mypy errors" {
        mypy
    }
}

task generator -depends init {
    Build-PyQIR($pyqir.generator.name)
}

task evaluator -depends init {
    Build-PyQIR($pyqir.evaluator.name)
}

task parser -depends init {
    Build-PyQIR($pyqir.parser.name)
}

task pyqir-tests -depends init, generator, evaluator {
    exec -workingDirectory $pyqir.tests.dir {
        pytest
    }
}

task qirlib -depends init {
    if (Test-MuslLinux) {
        # https://github.com/rust-lang/rust/issues/71651
        $old_rustflags = ""
        $reset_rustflags = $false

        if (Test-Path env:\RUSTFLAGS) {
            $reset_rustflags = $true
            $old_rustflags = $env:RUSTFLAGS
            $env:RUSTFLAGS = "$($old_rustflags) -C target-feature=-crt-static".Trim()
        }
        else {
            $env:RUSTFLAGS = "-C target-feature=-crt-static"
        }
        try {
            Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
                cargo test --release @("$($env:CARGO_EXTRA_ARGS)" -split " ")
            }
        }
        finally {
            if ($reset_rustflags) {
                $env:RUSTFLAGS = $old_rustflags
            }
            else {
                Remove-Item env:\RUSTFLAGS
            }
        }
    }
    else {
        Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
            cargo test --release @("$($env:CARGO_EXTRA_ARGS)" -split " ")
        }
    }
    Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
        cargo build --release @("$($env:CARGO_EXTRA_ARGS)" -split " ")
    }
}

task metawheel {
    $wheelDir = Split-Path -Parent $wheelhouse
    if (!(Test-Path $wheelDir)) {
        New-Item -Path $wheelDir -ItemType Directory | Out-Null
    }
    Invoke-LoggedCommand {
        & $python -m pip wheel --no-deps --wheel-dir $wheelDir "$($pyqir.meta.dir)"
    }
}

task wheelhouse `
    -precondition { -not (Test-Path $wheelhouse -ErrorAction SilentlyContinue) } `
{ Invoke-Task rebuild }

task docs -depends wheelhouse {
    # Write out the wheels available
    Write-Host (Get-ChildItem $wheelhouse -Include *.whl)

    # - Install artifacts into new venv along with sphinx.
    # - Run sphinx from within new venv.
    $envPath = Join-Path $repo.root ".docs-venv"
    $sphinxOpts = $docs.build.opts
    Create-PyEnv `
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

task check-environment {
    $env_message = @(
        "PyQIR requires a virtualenv or conda environment to build.",
        "Neither the VIRTUAL_ENV nor CONDA_PREFIX environment variables are set).",
        "See https://virtualenv.pypa.io/en/latest/index.html on how to use virtualenv"
    )
    if ((Test-InVirtualEnvironment) -eq $false) {
        Write-BuildLog "No virtual environment found."
        $pyenv = Join-Path $repo.target ".env"
        Write-BuildLog "Setting up virtual environment in $($pyenv)"
        & $python -m venv $pyenv
        if ($IsWindows) {
            . (Join-Path $pyenv "Scripts" "Activate.ps1")
        }
        else {
            . (Join-Path $pyenv "bin" "Activate.ps1")
        }
    }
    else {
        Write-BuildLog "Virtual environment found."
    }
    # ensure that we are now in a virtual environment
    Assert ((Test-InVirtualEnvironment) -eq $true) "$($env_message -join ' ')"
}

task python-requirements -depends check-environment {
    pip install `
        --requirement (Join-Path $pyqir.generator.examples_dir requirements.txt) `
        --requirement (Join-Path $pyqir.evaluator.dir requirements-dev.txt) `
        --requirement (Join-Path $pyqir.generator.dir requirements-dev.txt) `
        --requirement (Join-Path $pyqir.parser.dir requirements-dev.txt)
}

task init -depends python-requirements {
    if (Test-CI) {
        & $python -m pip install maturin==0.12.12
    }

    $env:CARGO_EXTRA_ARGS = "-vv --features `"$(Get-LLVMFeatureVersion)`""

    # qirlib has this logic built in when compiled on its own
    # but we must have LLVM installed prior to the wheels being built.
    
    # if an external LLVM is specified, make sure it exist and
    # skip further bootstapping
    if (Test-Path env:\QIRLIB_LLVM_EXTERNAL_DIR) {
        Use-ExternalLlvmInstallation
    }
    else {
        $packagePath = Resolve-InstallationDirectory
        if (Test-LlvmConfig $packagePath) {
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
            $installationDirectory = Resolve-InstallationDirectory
            Use-LlvmInstallation $installationDirectory
        }
    }
}

task install-llvm-from-archive {
    install-llvm $pyqir.qirlib.dir "download"
    $installationDirectory = Resolve-InstallationDirectory
    Assert (Test-LlvmConfig $installationDirectory) "install-llvm-from-archive failed to install a usable LLVM installation"
}


task install-llvm-from-source -depends configure-sccache -postaction { Write-CacheStats } {
    if ($IsWindows) {
        Include vcvars.ps1
    }
    install-llvm "$($pyqir.qirlib.dir)" "build" "$(Get-LLVMFeatureVersion)"
    $installationDirectory = Resolve-InstallationDirectory
    Assert (Test-LlvmConfig $installationDirectory) "install-llvm-from-source failed to install a usable LLVM installation"
}

task package-musllinux-llvm -depends build-musllinux-container-image -preaction { Write-CacheStats } -postaction { Write-CacheStats } {
    $srcPath = $repo.root

    # For any of the volumes mapped, if the dir doesn't exist,
    # docker will create it and it will be owned by root and
    # the caching/install breaks with permission errors.
    # New-Item is idempotent so we don't need to check for existence

    $cacheMount, $cacheEnv = Get-CCacheParams

    Write-BuildLog "Running container image: $($linux.musllinux_tag)"
    $ioVolume = "$($srcPath):$($linux.musllinux_root)"
    $userName = Get-LinuxContainerUserName
    if (Test-CI) {
        $userName = "root"
    }
    Invoke-LoggedCommand {
        docker run --rm --user $userName -v $ioVolume @cacheMount @cacheEnv -w "$($linux.musllinux_root)" -e QIRLIB_PKG_DEST="$($linux.musllinux_root)/target/musllinux" "$($linux.musllinux_tag)" pwsh build.ps1 -t package-llvm
    }
}

task package-manylinux-llvm -depends build-manylinux-container-image -preaction { Write-CacheStats } -postaction { Write-CacheStats } {
    $srcPath = $repo.root

    # For any of the volumes mapped, if the dir doesn't exist,
    # docker will create it and it will be owned by root and
    # the caching/install breaks with permission errors.
    # New-Item is idempotent so we don't need to check for existence

    $cacheMount, $cacheEnv = Get-CCacheParams

    Write-BuildLog "Running container image: $($linux.manylinux_tag)"
    $ioVolume = "$($srcPath):$($linux.manylinux_root)"
    $userName = Get-LinuxContainerUserName

    Invoke-LoggedCommand {
        docker run --rm --user $userName -v $ioVolume @cacheMount @cacheEnv -w "$($linux.manylinux_root)" -e QIRLIB_PKG_DEST="$($linux.manylinux_root)/target/manylinux" "$($linux.manylinux_tag)" conda run --no-capture-output pwsh build.ps1 -t package-llvm
    }
}

task package-llvm {
    if ($IsWindows) {
        Include vcvars.ps1
    }
    $clear_pkg_dest_var = $false
    if (!(Test-Path env:\QIRLIB_PKG_DEST)) {
        $clear_pkg_dest_var = $true
        $env:QIRLIB_PKG_DEST = Join-Path $repo.root "target"
    }
    New-Item $env:QIRLIB_PKG_DEST -ItemType Directory -Force
    try {
        Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
            cargo build --release --no-default-features --features "package-llvm,$(Get-LLVMFeatureVersion)-no-llvm-linking" -vv
        }
    }
    finally {
        if ($clear_pkg_dest_var) {
            Remove-Item -Path Env:QIRLIB_PKG_DEST
        }
    }
}

task build-manylinux-container-image {
    $srcPath = $repo.root
    Write-BuildLog "Building container image manylinux-llvm-builder"
    Invoke-LoggedCommand -workingDirectory (Join-Path $srcPath eng) {
        $user = "$(Get-LinuxContainerUserName)"
        $uid = "$(Get-LinuxContainerUserId)"
        $gid = "$(Get-LinuxContainerGroupId)"
        $rustv = "$($rust.version)"
        $tag = "$($linux.manylinux_tag)"
        Get-Content manylinux.Dockerfile | docker build `
            --build-arg USERNAME=$user `
            --build-arg USER_UID=$uid `
            --build-arg USER_GID=$gid `
            --build-arg RUST_VERSION=$rustv `
            -t $tag -
    }
}

task build-musllinux-container-image {
    $srcPath = $repo.root
    Write-BuildLog "Building container image musllinux-llvm-builder"
    if (Test-CI) {
        Invoke-LoggedCommand -workingDirectory (Join-Path $srcPath eng) {
            $rustv = "$($rust.version)"
            $tag = "$($linux.musllinux_tag)"
            Get-Content musllinuxCI.Dockerfile | docker build `
                --build-arg RUST_VERSION=$rustv `
                -t $tag -
        }
    }
    else {
        Invoke-LoggedCommand -workingDirectory (Join-Path $srcPath eng) {
            $user = "$(Get-LinuxContainerUserName)"
            $uid = "$(Get-LinuxContainerUserId)"
            $gid = "$(Get-LinuxContainerGroupId)"
            $rustv = "$($rust.version)"
            $tag = "$($linux.musllinux_tag)"
            Get-Content musllinux.Dockerfile | docker build `
                --build-arg USERNAME=$user `
                --build-arg USER_UID=$uid `
                --build-arg USER_GID=$gid `
                --build-arg RUST_VERSION=$rustv `
                -t $tag -
        }
    }
}

# This is only usable if building for manylinux
task run-examples-in-containers {
    $user = Get-LinuxContainerUserName
    $uid = "$(Get-LinuxContainerUserId)"
    $gid = "$(Get-LinuxContainerGroupId)"
    $images = @("buster", "bullseye", "bionic", "focal")
    foreach ($image in $images) {
        exec -workingDirectory (Join-Path $repo.root "eng") {
            get-content "$($image).Dockerfile" | docker build --build-arg USERNAME=$user --build-arg USER_UID=$uid --build-arg USER_GID=$gid -t "pyqir-$image-examples" -
        }
        exec {
            docker run --rm --user $user -v "$($repo.root):/home/$user" "pyqir-$image-examples" build.ps1 -t run-examples
        }
    }
}

task run-examples-in-musl-containers {
    $userName = Get-LinuxContainerUserName
    if (Test-CI) {
        $userName = "root"
    }
    Invoke-LoggedCommand {
        docker run --rm --user $userName -w "/home/$user" -v "$($repo.root):/home/$user" "$($linux.musllinux_tag)" pwsh build.ps1 -t run-examples
    }
}

# run-examples assumes the wheels have already been installed locally
task run-examples {   
    exec -workingDirectory $pyqir.generator.examples_dir {
        & $python -m pip install -U pip wheel
        & $python -m pip install -r requirements.txt
        & $python -m pip install -U --find-links (Join-Path $repo.root "target" "wheels") pyqir-generator

        & $python "bell_pair.py" | Tee-Object -Variable output
        $head = $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'Bell'") "bell_pair.py doesn't print expected module ID."

        $output = Join-Path $($env:TEMP) "bz.ll"
        & $python "mock_to_qir.py" -o $output "bernstein_vazirani.txt" 7
        $head = Get-Content $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'bernstein_vazirani'") "mock_to_qir.py doesn't print expected module ID."

        & $python "if_result.py" | Tee-Object -Variable output
        $head = $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'if_result'") "if_result.py doesn't print expected module ID."

        & $python "if_bool.py" | Tee-Object -Variable output
        $head = $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'if_bool'") "if_bool.py doesn't print expected module ID."

        & $python "external_functions.py" | Tee-Object -Variable output
        $head = $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'external_functions'") "external_functions.py doesn't print expected module ID."

        & $python "arithmetic.py" | Tee-Object -Variable output
        $head = $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'arithmetic'") "arithmetic.py doesn't print expected module ID."

        & $python "dynamic_allocation.py" | Tee-Object -Variable output
        $head = $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'dynamic_allocation'") "dynamic_allocation.py doesn't print expected module ID."
    }

    exec -workingDirectory $pyqir.evaluator.examples_dir {
        & $python -m pip install -U --no-index --find-links (Join-Path $repo.root "target" "wheels") pyqir-evaluator
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
   
    exec -workingDirectory $pyqir.evaluator.examples_dir {
        & $python "teleport.py" | Tee-Object -Variable teleport_output
        $teleport_first_lines = @($teleport_output | Select-Object -first 5)
        $teleport_expected = @(
            "# Evaluating both results as 0's",
            "qubits[3]",
            "out[3]",
            "h qubit[2]",
            "cx qubit[2], qubit[1]"
        )
        Assert (@(Compare-Object $teleport_first_lines $teleport_expected).Length -eq 0) "Expected $teleport_expected found $teleport_first_lines"
    }
}

task check-licenses {
    # Uses cargo-deny to verify that the linked components
    # only use approved licenses
    # https://github.com/EmbarkStudios/cargo-deny
    Invoke-LoggedCommand -wd $repo.root {
        cargo deny check licenses
    }
}

task update-noticefiles {
    # use cargo-about to generate a notice files
    # notice files are only for wheel distributions
    # as no bundled sources are in the sdist.

    # llvm special license is already in the template
    # as it is a hidden transitive dependency.
    # https://github.com/EmbarkStudios/cargo-about
    $config = Join-Path $repo.root notice.toml
    $template = Join-Path $repo.root notice.hbs
    foreach ($project in @($pyqir.parser.dir, $pyqir.generator.dir, $pyqir.evaluator.dir)) {
        Invoke-LoggedCommand -wd $project {
            $notice = Join-Path $project NOTICE-WHEEL.txt
            cargo about generate --config $config --all-features --output-file $notice $template
            $contents = Get-Content -Raw $notice
            [System.Web.HttpUtility]::HtmlDecode($contents) | Out-File $notice
        }
    }
}

task configure-sccache -postaction { Write-CacheStats } {
    if (Test-CommandExists("sccache")) {
        Write-BuildLog "Starting sccache server"
        & { sccache --start-server } -ErrorAction SilentlyContinue
        Write-BuildLog "Started sccache server"
    }
}
