# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Include utils.ps1

Properties {
    $Root = Resolve-Path (Split-Path -Parent $PSScriptRoot)
    $Qirlib = Join-Path $Root qirlib
    $Pyqir = Join-Path $Root pyqir
    $Examples = Join-Path $Root examples
    $Target = Join-Path $Root target
    $Wheels = Join-Path $Target wheels
    $CargoConfigToml = Join-Path $Root .cargo config.toml
    $VscodeSettingsJson = Join-Path $Root .vscode settings.json
    $DocsRoot = Join-Path $Root docs
    $DocsBuild = Join-Path $DocsRoot _build
    $Python = Resolve-Python
}

task default -depends build, test, run-examples
task build -depends qirlib, pyqir
task checks -depends cargo-fmt, cargo-clippy, black, mypy

task cargo-fmt {
    Invoke-LoggedCommand -workingDirectory $Root -errorMessage "Please run 'cargo fmt --all' before pushing" {
        cargo fmt --all -- --check
    }
}

task cargo-clippy -depends init {
    Invoke-LoggedCommand -workingDirectory $Root -errorMessage "Please fix the above clippy errors" {
        cargo clippy --workspace --all-targets @(Get-CliCargoArgs) -- -D warnings
    }
}

task black -depends check-environment {
    exec { & $Python -m pip install black }
    Invoke-LoggedCommand -workingDirectory $Root -errorMessage "Please run black before pushing" {
        & $Python -m black --check --extend-exclude "^/examples/mock_language/" .
    }
}

task mypy -depends check-environment {
    $reqs = Resolve-PythonRequirements "$Pyqir[test]"
    exec { & $Python -m pip install --requirement (Join-Path $Examples requirements.txt) @reqs mypy }
    Invoke-LoggedCommand -workingDirectory $Root -errorMessage "Please fix the above mypy errors" {
        mypy
    }
}

task qirlib -depends init {
    Invoke-LoggedCommand -workingDirectory $Qirlib { cargo test --release @(Get-CliCargoArgs) }
    Invoke-LoggedCommand -workingDirectory $Qirlib { cargo build --release @(Get-CliCargoArgs) }
}

task pyqir -depends init {
    $configSettings = @(Get-CliCargoArgs) -Join " "
    Get-Wheels pyqir | Remove-Item

    Invoke-LoggedCommand { & $Python -m pip --verbose wheel --config-settings=build-args="$configSettings" --wheel-dir $Wheels $Pyqir }

    if ($IsLinux) {
        Invoke-LoggedCommand { & $Python -m pip install auditwheel patchelf }
    }
    if (Test-CommandExists auditwheel) {
        $unauditedWheels = Get-Wheels pyqir
        Invoke-LoggedCommand { auditwheel show $unauditedWheels }
        Invoke-LoggedCommand { auditwheel repair --wheel-dir $Wheels --plat (Get-AuditWheelTag($Python)) $unauditedWheels }
        $unauditedWheels | Remove-Item
    }
}

task test {
    $packages = Get-Wheels pyqir | ForEach-Object { "$_[test]" }
    Invoke-LoggedCommand { & $Python -m pip install --force-reinstall $packages }
    Invoke-LoggedCommand -workingDirectory $Pyqir { pytest }
}

task wheelhouse -precondition { -not (Test-Path (Join-Path $Wheels *.whl)) } {
    Invoke-Task build
}

task docs -depends check-environment, wheelhouse {
    Invoke-LoggedCommand {
        & $Python -m pip install --requirement (Join-Path $DocsRoot requirements.txt) (Join-Path $Wheels *.whl)
    }
    Invoke-LoggedCommand { sphinx-build -M html $DocsRoot $DocsBuild -W --keep-going }
}

task check-environment {
    $env_message = @(
        "PyQIR requires a virtualenv or conda environment to build.",
        "Neither the VIRTUAL_ENV nor CONDA_PREFIX environment variables are set.",
        "See https://virtualenv.pypa.io/en/latest/index.html on how to use virtualenv"
    )

    if ((Test-InVirtualEnvironment) -eq $false) {
        Write-BuildLog "No virtual environment found."
        $pyenv = Join-Path $Target ".env"
        Write-BuildLog "Setting up virtual environment in $pyenv"
        & $Python -m venv $pyenv
        if ($IsWindows) {
            . (Join-Path $pyenv Scripts Activate.ps1)
        }
        else {
            . (Join-Path $pyenv bin Activate.ps1)
        }
    }
    else {
        Write-BuildLog "Virtual environment found."
    }

    Assert ((Test-InVirtualEnvironment) -eq $true) ($env_message -Join ' ')
    exec { & $Python -m pip install -U pip }
}

task init -depends check-environment {
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
    install-llvm $Qirlib download
    $installationDirectory = Resolve-InstallationDirectory
    Assert (Test-LlvmConfig $installationDirectory) "install-llvm-from-archive failed to install a usable LLVM installation"
}


task install-llvm-from-source -depends configure-sccache -postaction { Write-CacheStats } {
    if ($IsWindows) {
        Include vcvars.ps1
    }
    install-llvm $Qirlib build (Get-LLVMFeatureVersion)
    $installationDirectory = Resolve-InstallationDirectory
    Assert (Test-LlvmConfig $installationDirectory) "install-llvm-from-source failed to install a usable LLVM installation"
}

task package-llvm {
    if ($IsWindows) {
        Include vcvars.ps1
    }
    $clear_pkg_dest_var = $false
    if (!(Test-Path env:\QIRLIB_PKG_DEST)) {
        $clear_pkg_dest_var = $true
        $env:QIRLIB_PKG_DEST = $Target
    }
    New-Item $env:QIRLIB_PKG_DEST -ItemType Directory -Force
    try {
        Invoke-LoggedCommand -workingDirectory $Qirlib {
            cargo build --release --no-default-features --features "package-llvm,$(Get-LLVMFeatureVersion)-no-llvm-linking" -vv
        }
    }
    finally {
        if ($clear_pkg_dest_var) {
            Remove-Item -Path Env:QIRLIB_PKG_DEST
        }
    }
}

# run-examples assumes the wheels have already been installed locally
task run-examples {
    exec -workingDirectory $Examples {
        & $Python -m pip install --requirement requirements.txt --use-pep517
        & $Python -m pip install --force-reinstall (Get-Wheel pyqir)

        & $Python bell_pair.py | Tee-Object -Variable output
        $head = $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'Bell'") "bell_pair.py doesn't print expected module ID."

        $output = Join-Path $env:TEMP bz.ll
        & $Python mock_to_qir.py -o $output bernstein_vazirani.txt 7
        $head = Get-Content $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'bernstein_vazirani'") "mock_to_qir.py doesn't print expected module ID."

        & $Python if_result.py | Tee-Object -Variable output
        $head = $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'if_result'") "if_result.py doesn't print expected module ID."

        & $Python if_bool.py | Tee-Object -Variable output
        $head = $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'if_bool'") "if_bool.py doesn't print expected module ID."

        & $Python external_functions.py | Tee-Object -Variable output
        $head = $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'external_functions'") "external_functions.py doesn't print expected module ID."

        & $Python arithmetic.py | Tee-Object -Variable output
        $head = $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'arithmetic'") "arithmetic.py doesn't print expected module ID."

        & $Python dynamic_allocation.py | Tee-Object -Variable output
        $head = $output | Select-Object -First 1
        Assert ($head -eq "; ModuleID = 'dynamic_allocation'") "dynamic_allocation.py doesn't print expected module ID."

        & $Python bernstein_vazirani.py | Tee-Object -Variable bz_output
        $bz_lines = $bz_output -join ", "
        $bz_expected = "x(5), h(0), h(1), h(2), h(3), h(4), h(5), cnot(1, 5), cnot(3, 5), cnot(4, 5), h(0), h(1), h(2), h(3), h(4), mz(0, 0), mz(1, 1), mz(2, 2), mz(3, 3), mz(4, 4)"

        Assert (@(Compare-Object $bz_lines $bz_expected).Length -eq 0) "Expected $bz_expected found $bz_lines"

        & $Python teleport.py | Tee-Object -Variable teleport_output
        $teleport_first_lines = @($teleport_output | Select-Object -first 2)
        $teleport_expected = @(
            "; ModuleID = 'teleport'",
            "source_filename = `"teleport`""
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
    $config = Join-Path $Root notice.toml
    $template = Join-Path $Root notice.hbs
    $notice = Join-Path $Pyqir NOTICE-WHEEL.txt
    Invoke-LoggedCommand -workingDirectory $Pyqir {
        cargo about generate --config $config --all-features --output-file $notice $template
        $contents = Get-Content -Raw $notice
        [System.Web.HttpUtility]::HtmlDecode($contents) | Out-File $notice
    }
}

task configure-sccache -postaction { Write-CacheStats } {
    if (Test-CommandExists sccache) {
        Write-BuildLog "Starting sccache server"
        & { sccache --start-server } -ErrorAction SilentlyContinue
        Write-BuildLog "Started sccache server"
    }
}
