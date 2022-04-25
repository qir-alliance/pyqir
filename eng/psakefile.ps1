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

Include settings.ps1
Include utils.ps1

Task default -Depends qirlib, pyqir-tests, parser, generator, evaluator, metawheel, run-examples

Task manylinux -Depends Build-ManyLinuxContainerImage, Run-ManyLinuxContainerImage, run-examples-in-containers 

Task musllinux -Depends Build-MuslLinuxContainerImage, Run-MuslLinuxContainerImage

Task Run-ManyLinuxContainerImage -PreAction { Write-CacheStats } -PostAction { Write-CacheStats } {
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

Task Run-MuslLinuxContainerImage -PreAction { Write-CacheStats } -PostAction { Write-CacheStats } {
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
                & $python -m tox -v -e all
            }
        }
    }
}

Task qirlib -Depends init {
    if ($IsLinux) {
        $triple = Get-LinuxTargetTriple
        if ($triple -eq "x86_64-unknown-linux-musl") {
            $env:RUSTFLAGS = "-C target-feature=-crt-static"
        }
        Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
            cargo test --release -vv
        }
        $env:RUSTFLAGS = ""
        Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
            cargo build --release -vv
        }
    }
    else {
        Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
            cargo test --release -vv
        }
        Invoke-LoggedCommand -wd $pyqir.qirlib.dir {
            cargo build --release -vv
        }
    }
}

Task metawheel {
    $wheelDir = Split-Path -Parent $wheelhouse
    if (!(Test-Path $wheelDir)) {
        New-Item -Path $wheelDir -ItemType Directory | Out-Null
    }
    Invoke-LoggedCommand {
        & $python -m pip wheel --no-deps --wheel-dir $wheelDir "$($pyqir.meta.dir)"
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
    ((Test-Path env:\QIRLIB_DOWNLOAD_LLVM) -and ($env:QIRLIB_DOWNLOAD_LLVM -eq $true))
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

task install-llvm-from-source -Depends Configure-SCCache -PostAction { Write-CacheStats } {
    if ($IsWindows) {
        Include vcvars.ps1
    }
    install-llvm $pyqir.qirlib.dir "build"
}

task Package-MuslLinuxLLVM -Depends Build-MuslLinuxContainerImage -PreAction { Write-CacheStats } -PostAction { Write-CacheStats } {
    if ($IsLinux) {
        $srcPath = $repo.root
        $ioVolume = "$($srcPath):$($linux.musllinux_root)"
        $userName = Get-LinuxContainerUserName

        Invoke-LoggedCommand {
            docker run --rm --user $userName -v $ioVolume -w "$($linux.musllinux_root)/qirlib" -e QIRLIB_PKG_DEST="$($linux.musllinux_root)/target" "$($linux.musllinux_tag)" cargo build --release --no-default-features --features package-llvm -vv
        }
    }
}

task Package-ManyLinuxLLVM -Depends Build-ManyLinuxContainerImage -PreAction { Write-CacheStats } -PostAction { Write-CacheStats } {
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

task Build-ManyLinuxContainerImage {
    $srcPath = $repo.root
    Write-BuildLog "Building container image manylinux-llvm-builder"
    Invoke-LoggedCommand -workingDirectory (Join-Path $srcPath eng) {
        $user = "$(Get-LinuxContainerUserName)"
        $uid = "$(Get-LinuxContainerUserId)"
        $gid = "$(Get-LinuxContainerGroupId)"
        $rustv = "1.57.0"
        $tag = "$($linux.manylinux_tag)"
        Get-Content manylinux.Dockerfile | docker build `
            --build-arg USERNAME=$user `
            --build-arg USER_UID=$uid `
            --build-arg USER_GID=$gid `
            --build-arg RUST_VERSION=$rustv `
            -t $tag -
    }
}

task Build-MuslLinuxContainerImage {
    $srcPath = $repo.root
    Write-BuildLog "Building container image musllinux-llvm-builder"
    if (Test-CI) {
        Invoke-LoggedCommand -workingDirectory (Join-Path $srcPath eng) {
            $rustv = "1.57.0"
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
            $rustv = "1.57.0"
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

function Build-PyQIR([string]$project) {
    $srcPath = $repo.root

    exec -workingDirectory (Join-Path $srcPath $project) {
        if (Test-InCondaEnvironment) {
            Invoke-LoggedCommand {
                maturin build --release --cargo-extra-args="-vv"
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
                & $python -m tox -v -e all
            }
        }
    }
}

# This is only usable if building for manylinux
Task run-examples-in-containers {
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

# run-examples assumes the wheels have already been installed locally
task run-examples {   
    exec -workingDirectory $pyqir.generator.examples_dir {
        & $python -m pip install -r requirements.txt
        & $python -m pip install --no-index --find-links (Join-Path $repo.root "target" "wheels") pyqir-generator
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
        & $python -m pip install --no-index --find-links (Join-Path $repo.root "target" "wheels") pyqir-evaluator
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

Task Configure-SCCache -PostAction { Write-CacheStats } {
    if (Test-CommandExists("sccache")) {
        Write-BuildLog "Starting sccache server"
        & { sccache --start-server } -ErrorAction SilentlyContinue
        Write-BuildLog "Started sccache server"
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
