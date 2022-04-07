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

    $wheelhouse = Join-Path $repo.root "target" "wheels" "*.whl"
}

Include settings.ps1
Include utils.ps1

Task default -Depends checks, pyqir-tests, parser, generator, evaluator, metawheel, run-examples, run-examples-in-containers

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
    $project = "pyqir-tests"
    $installationDirectory = Resolve-InstallationDirectory

    if (Test-RunInContainer) {
        Build-ContainerImage $srcPath
        Write-BuildLog "Running container image:"
        $ioVolume = "$($srcPath):/io"
        $llvmVolume = "$($installationDirectory):/usr/lib/llvm"
        $userName = [Environment]::UserName

        Invoke-LoggedCommand {
            docker run --rm --user $userName -v $ioVolume -v $llvmVolume -e LLVM_SYS_110_PREFIX=/usr/lib/llvm -w /io/$project manylinux2014_x86_64_maturin conda run --no-capture-output python -m tox -e test
        }
    }
    else {
        exec -workingDirectory (Join-Path $srcPath $project) {
            Invoke-LoggedCommand {
                & $python -m pip install tox
            }

            Invoke-LoggedCommand {
                & $python -m tox -e test
            }
        }
    }
}

Task metawheel {
    $wheelDir = Split-Path -Parent $wheelhouse
    if (!(Test-Path $wheelDir)) {
        New-Item -Path $wheelDir -ItemType Directory | Out-Null
    }
    Invoke-LoggedCommand {
        & $python -m pip wheel --no-deps --wheel-dir $wheelDir $pyqir.meta.dir
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
    # Temorary, install maturin v0.12.12-beta.2 which has the
    # PEP 639 license fixes.
    if ((Test-CI) -and !$IsLinux) {
        cargo install maturin --git https://github.com/PyO3/maturin --tag v0.12.12-beta.2
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

task install-llvm-from-source {
    Write-CacheStats
    if (Test-CommandExists("sccache")) {
        Write-BuildLog "Starting sccache server"
        & { sccache --start-server } -ErrorAction SilentlyContinue
        Write-BuildLog "Started sccache server"
    }

    if (Test-RunInContainer) {
        $installationDirectory = Resolve-InstallationDirectory
        Use-LlvmInstallation $installationDirectory
        Build-ContainerImage $repo.root
        $srcPath = $repo.root

        # For any of the volumes mapped, if the dir doesn't exist,
        # docker will create it and it will be owned by root and
        # the caching/install breaks with permission errors.
        # New-Item is idempotent so we don't need to check for existence

        $ioVolume = "$($srcPath):/io"
        $llvmVolume = "$($installationDirectory):/llvm"
        New-Item -ItemType Directory -Force $installationDirectory | Out-Null
        
        $userName = [Environment]::UserName
        $cacheMount = ""
        $cacheEnv = ""
        # only ccache is supported in the manylinux container for now.
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
            }
        }
        
        Invoke-LoggedCommand {
            docker run --rm --user $userName -v $ioVolume -v $llvmVolume @cacheMount @cacheEnv -e QIRLIB_CACHE_DIR="$installationDirectory" -e QIRLIB_CACHE_DIR="/llvm" -w /io/qirlib manylinux2014_x86_64_maturin conda run --no-capture-output cargo build --release --no-default-features --features "build-llvm,no-llvm-linking" -vv
        }
    }
    else {
        if ($IsWindows) {
            Include vcvars.ps1
        }
        
        install-llvm $pyqir.qirlib.dir "build"
    }

    Write-CacheStats
}

task package-llvm {
    if (Test-RunInContainer) {
        Build-ContainerImage $repo.root
        $srcPath = $repo.root
        $ioVolume = "$($srcPath):/io"
        $userName = [Environment]::UserName

        Invoke-LoggedCommand {
            docker run --rm --user $userName -v $ioVolume -w /io/qirlib -e QIRLIB_PKG_DEST=/io/target manylinux2014_x86_64_maturin conda run --no-capture-output cargo build --release  --no-default-features --features package-llvm -vv
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
                cargo build --release  --no-default-features --features package-llvm -vv
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
        $user = [environment]::UserName
        $uid = "$(id -u)"
        $gid = "$(id -g)"
        $rustv = "1.57.0"
        $tag = "manylinux2014_x86_64_maturin"
        Get-Content manylinux.Dockerfile | docker build `
            --build-arg USERNAME=$user `
            --build-arg USER_UID=$uid `
            --build-arg USER_GID=$gid `
            --build-arg RUST_VERSION=$rustv `
            -t $tag -
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
            $userName = [Environment]::UserName

            Invoke-LoggedCommand {
                docker run --rm --user $userName -v $ioVolume -v $llvmVolume -e LLVM_SYS_110_PREFIX=/usr/lib/llvm -w /io/$project manylinux2014_x86_64_maturin conda run --no-capture-output cargo test --release --lib -vv -- --nocapture
            }

            Invoke-LoggedCommand {
                docker run --rm --user $userName -v $ioVolume -v $llvmVolume -e LLVM_SYS_110_PREFIX=/usr/lib/llvm -w /io/$project manylinux2014_x86_64_maturin conda run --no-capture-output /usr/bin/maturin build --release
            }

            Invoke-LoggedCommand {
                docker run --rm --user $userName -v $ioVolume -v $llvmVolume -e LLVM_SYS_110_PREFIX=/usr/lib/llvm -w /io/$project manylinux2014_x86_64_maturin conda run --no-capture-output python -m tox -e test
            }
            
            Invoke-LoggedCommand {
                docker run --rm --user $userName -v $ioVolume -v $llvmVolume -e LLVM_SYS_110_PREFIX=/usr/lib/llvm -w /io/$project manylinux2014_x86_64_maturin conda run --no-capture-output cargo test --release -vv -- --nocapture
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

function install-llvm {
    Param(
        [Parameter(Mandatory)]
        [string]$qirlibDir,
        [Parameter(Mandatory)]
        [ValidateSet("download", "build")]
        [string]$operation
    )

    $installationDirectory = Resolve-InstallationDirectory
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
