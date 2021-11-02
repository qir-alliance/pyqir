# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

$python = "python3"
if ($null -ne (Get-Command python -ErrorAction SilentlyContinue)) {
    $pythonIsPython3 = (python --version) -match "Python 3.*"
    if ($pythonIsPython3) {
        $python = "python"
    }
}

properties {
    $python = $python
}