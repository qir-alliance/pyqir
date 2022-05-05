# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

properties {
    $repo = @{}
    $repo.root = Resolve-Path (Split-Path -parent $PSScriptRoot)
    $repo.target = Join-Path $repo.root "target"

    $pyqir = @{}

    $pyqir.qirlib = @{}
    $pyqir.qirlib.name = "qirlib"
    $pyqir.qirlib.dir = Join-Path $repo.root "qirlib"

    $pyqir.meta = @{}
    $pyqir.meta.name = "pyqir"
    $pyqir.meta.dir = Join-Path $repo.root "pyqir"

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

    $rust = @{}
    $rust.version = "1.57.0"

    $linux = @{}
    $linux.manylinux_tag = "manylinux2014_x86_64_maturin"
    $linux.manylinux_root = "/io"
    $linux.musllinux_tag = "musllinux_1_2_x86_64_maturin"
    $linux.musllinux_root = "/oi"

    $wheelhouse = Join-Path $repo.root "target" "wheels" "*.whl"
}

include settings.ps1
include utils.ps1

task default -depends qirlib, pyqir-tests, parser, generator, evaluator, metawheel, run-examples

task manylinux -depends build-manylinux-container-image, run-manylinux-container-image, run-examples-in-containers 

task musllinux -depends build-musllinux-container-image, run-musllinux-container-image, run-examples-in-musl-containers

task checks -depends cargo-fmt, cargo-clippy

task rebuild -depends generator, evaluator, parser

task run-manylinux-container-image -preaction { Write-CacheStats } -postaction { Write-CacheStats } {
    $srcPath = $repo.root

    # For any of the volumes mapped, if the dir doesn't exist,
    # docker will create it and it will be owned by root and
    # the caching/install breaks with permission errors.
    # New-Item is idempotent so we don't need to check for existence

    $cacheMount, $cacheEnv = Get-CCacheParams

    Write-BuildLog "Running container image:"
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

    Write-BuildLog "Running container image:"
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
        $extraArgs = (Test-CI) ? @("--", "-D", "warnings") : @()
        cargo clippy --workspace --all-targets @extraArgs
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

task pyqir-tests -depends init {
    $srcPath = $repo.root

    exec -workingDirectory (Join-Path $srcPath "pyqir-tests") {
        if (Test-InCondaEnvironment) {
            Invoke-LoggedCommand -wd $pyqir.generator.dir {
                maturin develop --release --cargo-extra-args="-vv"
            }
            Invoke-LoggedCommand -wd $pyqir.evaluator.dir {
                maturin develop --release --cargo-extra-args="-vv"
            }
            & $python -m pip install pytest
            & $python -m pytest
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
        try {
            Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
                cargo test --release -vv
            }
        }
        finally {
            if ($reset_rustflags) {
                $env:RUSTFLAGS = $old_rustflags
            }
        }
    }
    else {
        Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
            cargo test --release -vv
        }
    }
    Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
        cargo build --release -vv
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

task init {
    if ((Test-CI)) {
        cargo install maturin --git https://github.com/PyO3/maturin --tag v0.12.12
    }
    
    # qirlib has this logic built in when compiled on its own
    # but we must have LLVM installed prior to the wheels being built.
    
    # if an external LLVM is specified, make sure it exist and
    # skip further bootstapping
    if (Test-Path env:\QIRLIB_LLVM_EXTERNAL_DIR) {
        Use-ExternalLlvmInstallation
    }
    else {
        $packagePath = Resolve-InstallationDirectory
        if (Test-Path (Join-Path $packagePath "bin")) {
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
    install-llvm $pyqir.qirlib.dir "download"
}


task install-llvm-from-source -depends configure-sccache -postaction { Write-CacheStats } {
    if ($IsWindows) {
        Include vcvars.ps1
    }
    install-llvm $pyqir.qirlib.dir "build"
}

task package-musllinux-llvm -depends build-musllinux-container-image -preaction { Write-CacheStats } -postaction { Write-CacheStats } {
    if ($IsLinux) {
        $srcPath = $repo.root
        $ioVolume = "$($srcPath):$($linux.musllinux_root)"
        $userName = Get-LinuxContainerUserName

        Invoke-LoggedCommand {
            docker run --rm --user $userName -v $ioVolume -w "$($linux.musllinux_root)/qirlib" -e QIRLIB_PKG_DEST="$($linux.musllinux_root)/target" "$($linux.musllinux_tag)" cargo build --release --no-default-features --features package-llvm -vv
        }
    }
}

task package-manylinux-llvm -depends build-manylinux-container-image -preaction { Write-CacheStats } -postaction { Write-CacheStats } {
    if ($IsLinux) {
        $srcPath = $repo.root
        $ioVolume = "$($srcPath):$($linux.manylinux_root)"
        $userName = Get-LinuxContainerUserName

        Invoke-LoggedCommand {
            docker run --rm --user $userName -v $ioVolume -w "$($linux.manylinux_root)/qirlib" -e QIRLIB_PKG_DEST="$($linux.manylinux_root)/target" "$($linux.manylinux_tag)" conda run --no-capture-output cargo build --release --no-default-features --features package-llvm -vv
        }
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
    try {
        Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
            cargo build --release  --no-default-features --features package-llvm -vv
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
        & $python -m pip install -U pip
        & $python -m pip install -r requirements.txt
        & $python -m pip install --user -U --no-index --find-links (Join-Path $repo.root "target" "wheels") pyqir-generator
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
        & $python -m pip install --user -U --no-index --find-links (Join-Path $repo.root "target" "wheels") pyqir-evaluator
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
