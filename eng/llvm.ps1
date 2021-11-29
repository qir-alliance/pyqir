# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Include settings.ps1
Include utils.ps1
if ($IsWindows) {
    Include vcvars.ps1
}

# Build

properties {
    $repo = @{}
    $repo.root = Resolve-Path (Split-Path -parent $PSScriptRoot)

    $buildDir = Join-Path $repo.root eng
    $llvmCmakeFile = Join-Path $buildDir llvm.cmake
    $llvmRootDir = Join-Path $repo.root external llvm-project
    $llvmBuildDir = Join-Path $llvmRootDir build
    $llvmDir = Join-Path $llvmRootDir llvm
    $PKG_NAME = Get-PackageName

    $package = @{}
    $package.Name = Get-PackageName
    $package.Path = Get-InstallationDirectory $package.Name
    $INSTALL_LLVM_PACKAGE = $false
}

Task default -Depends build, test-packages

Task init-caches {
    if (Test-CommandExists ccache) {
        Write-BuildLog "Found ccache command"
    
        if (-not (Test-Path env:\CCACHE_DIR)) {
            # CCACHE_DIR needs to be set, get the value from ccache
            $Env:CCACHE_DIR = exec { ccache -k cache_dir }
        }
        Assert (![string]::IsNullOrWhiteSpace($Env:CCACHE_DIR)) "CCACHE_DIR is not set"
    
        # Set cap and make sure dir is created
        if (!(Test-Path $Env:CCACHE_DIR)) {
            mkdir $Env:CCACHE_DIR | Out-Null
        }
        $Env:CCACHE_DIR = Resolve-Path $Env:CCACHE_DIR
        ccache -M 2G
    
        Write-BuildLog "Found ccache config:"
        ccache --show-config

        Write-BuildLog "Using CCACHE_DIR: $($Env:CCACHE_DIR)"
    }
    elseif (Test-CommandExists sccache) {
        Write-BuildLog "Found sccache command"
        # Set cap and make sure dir is created
        if ((Test-Path Env:\SCCACHE_DIR)) {
            $Env:SCCACHE_DIR = Resolve-Path $Env:SCCACHE_DIR
        
            if (!(Test-Path $Env:SCCACHE_DIR)) {
                mkdir $Env:SCCACHE_DIR | Out-Null
            }
        }
        $Env:SCCACHE_CACHE_SIZE = "2G"
        & { sccache --start-server } -ErrorAction SilentlyContinue
    }
    
    else {
        Write-BuildLog "Did not find ccache command"
    }
}

Task check-prerequisites {
    Assert (Test-Path $llvmDir) "llvm-project submodule is missing"
    Assert (![string]::IsNullOrWhiteSpace($PKG_NAME)) "PKG_NAME is not set"

    Assert (Test-CommandExists "cmake") "CMAKE not found"
    Assert (Test-CommandExists "ninja") "Ninja-Build not found"
    if ($IsLinux) {
        Assert (Test-CommandExists "docker") "Docker not found"
    }

    if (!(Test-Path $llvmBuildDir)) {
        mkdir $llvmBuildDir | Out-Null
    }
}

task build-containerimage -precondition { $IsLinux } {
    Write-BuildLog "Building container image manylinux-llvm-builder"
    Invoke-LoggedCommand -wd $buildDir {
        $userName = [Environment]::UserName
        $userId = $(id -u)
        $groupId = $(id -g)
        Write-BuildLog "Get-Content llvm.Dockerfile | docker build -t manylinux-llvm-builder --build-arg USERNAME=$userName --build-arg USER_UID=$userId --build-arg USER_GID=$groupId --build-arg LLVM_BUILD_DIR=""$llvmBuildDir"" -" "command"
        Get-Content llvm.Dockerfile | docker build -t manylinux-llvm-builder --build-arg USERNAME=$userName --build-arg USER_UID=$userId --build-arg USER_GID=$groupId --build-arg LLVM_BUILD_DIR="$llvmBuildDir" -
    }
}
 
Task build -Depends init-caches, check-prerequisites, build-containerimage {
    Write-BuildLog "Generating package: $($PKG_NAME)"

    if ($IsLinux) {
        # Verify input files/folders and mounts exist
        Assert (Test-Path $llvmCmakeFile) "llvmCmakeFile $($llvmCmakeFile) is missing"
        Assert (Test-Path $llvmDir) "llvmDir $($llvmDir) is missing"
        Assert (Test-Path $llvmBuildDir) "llvmBuildDir $($llvmBuildDir) is missing"

        Assert (Test-Path $repo.root) "repo.root $($repo.root) is missing"

        Write-BuildLog "Running container image:"
        $srcVolume = "$($repo.root):$($repo.root)"
        $cacheVolume = "$($Env:CCACHE_DIR):$($Env:CCACHE_DIR)"
        $cacheRoot = $Env:CCACHE_DIR
        $userSpec = [Environment]::UserName
            
        if ($true -eq $INSTALL_LLVM_PACKAGE) {
            Write-BuildLog "Installing package."
            if (!(Test-Path env:\CMAKE_INSTALL_PREFIX)) {
                Write-BuildLog "Using default package path"
                $env:CMAKE_INSTALL_PREFIX = $package.Path
            }
            Write-BuildLog "Setting up installation:"
            if (!(Test-Path $env:CMAKE_INSTALL_PREFIX)) {
                New-Item -ItemType Directory -Path $env:CMAKE_INSTALL_PREFIX -Force | Out-Null
            }
            $cmakeInstallVolume = "$($env:CMAKE_INSTALL_PREFIX):$($env:CMAKE_INSTALL_PREFIX)"
                    
            Invoke-LoggedCommand {
                docker run --rm --user $userSpec -e PKG_NAME=$($PKG_NAME) -e SOURCE_DIR=$($repo.root) -e LLVM_CMAKEFILE=$llvmCmakeFile -e LLVM_DIR=$llvmDir -e LLVM_BUILD_DIR=$llvmBuildDir -e CCACHE_DIR=$cacheRoot -e CCACHE_CONFIGPATH=$cacheRoot -v $srcVolume -v $cacheVolume -v $cmakeInstallVolume -e LLVM_INSTALL_DIR=$Env:CMAKE_INSTALL_PREFIX -e CMAKE_FLAGS="-DCMAKE_INSTALL_PREFIX=$($Env:CMAKE_INSTALL_PREFIX)" manylinux-llvm-builder
            }
                
        }
        else {
            Invoke-LoggedCommand {
                docker run --rm --user $userSpec -e PKG_NAME=$($PKG_NAME) -e SOURCE_DIR=$($repo.root) -e LLVM_CMAKEFILE=$llvmCmakeFile -e LLVM_DIR=$llvmDir -e LLVM_BUILD_DIR=$llvmBuildDir -e CCACHE_DIR=$cacheRoot -e CCACHE_CONFIGPATH=$cacheRoot -v $srcVolume -v $cacheVolume manylinux-llvm-builder
            }
        }
    }
    else {
        Invoke-LoggedCommand -workingDirectory $llvmBuildDir {
            Write-BuildLog "Generating makefiles"
            $flags = ""
            if (Test-Path env:\CMAKE_INSTALL_PREFIX) {
                $flags += "-DCMAKE_INSTALL_PREFIX=""$($Env:CMAKE_INSTALL_PREFIX)"""
            }
            cmake -G Ninja -C $llvmCmakeFile $flags $llvmDir
        }

        Invoke-LoggedCommand -workingDirectory $llvmBuildDir {
            ninja package
        }

        if ($true -eq $INSTALL_LLVM_PACKAGE) {
            Invoke-LoggedCommand -workingDirectory $llvmBuildDir {
                ninja install
            }
        }
    }
}

Task test-packages {
    Invoke-LoggedCommand -workingDirectory $llvmBuildDir {
        $package = Resolve-Path "$($PKG_NAME)*" -ErrorAction SilentlyContinue
        Assert ($null -ne $package) "Package is null"
        Assert (Test-Path $package) "Could not resolve package $package"
    }
}
