# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Write-BuildLog "Trying to load vcvars"
if ($IsWindows) {
    # find VS root
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (!(Test-Path $vswhere)) {
        throw "Could not locate vswhere.exe"
    }
    # vswhere doesn't include build tools only installs unless you specify -products *
    $visualStudioPath = & $vswhere -prerelease -latest -products * -property installationPath
    Write-BuildLog "vs located at: $visualStudioPath"
    $fileName = "vcvars64"
    if ($Env:PROCESSOR_ARCHITECTURE -eq "ARM64") {
        $fileName = "vcvarsarm64"
    }
    $varsBatPath = "$visualStudioPath\VC\Auxiliary\Build\$fileName.bat"
    if (!(Test-Path $varsBatPath)) {
        throw "Could not locate $fileName in $varsBatPath"
    }
    # Call vcvars<arm?>64.bat and write the set calls to file
    cmd.exe /c "call `"$varsBatPath`" && set > %temp%\vcvars.txt"

    # Read the set calls and set the corresponding pwsh env vars
    Get-Content "$Env:temp\vcvars.txt" | Foreach-Object {
        if ($_ -match "^(.*?)=(.*)$") {
            Set-Content "env:\$($matches[1])" $matches[2]
            Write-BuildLog "setting env: $($matches[1]) = $($matches[2])"
        }
    }
}