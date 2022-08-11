#!/usr/bin/env pwsh
$python = "python3"
if ($null -ne (Get-Command python -ErrorAction SilentlyContinue)) {
    $pythonIsPython3 = (python --version) -match "Python 3.*"
    if ($pythonIsPython3) {
        $python = "python"
    }
}
& $python -m pip install -r requirements.txt --no-index --find-links=wheelhouse -v
