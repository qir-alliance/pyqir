# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[string]$Version = "v1.13.1"
[string]$Sha256 = "fb959b674970e36a7c9a23191524b80fb5298fc71fc98bfa42456bcc0a8dfb2f"
[string]$ExtractPath = "C:\Program Files\Ninja"

$tempPath = $env:TEMP ?? [System.IO.Path]::GetTempPath()

# Variables
$downloadUrl = "https://github.com/ninja-build/ninja/releases/download/$Version/ninja-winarm64.zip"
$downloadPath = Join-Path $tempPath "ninja-winarm64.zip"
$ninjaExePath = Join-Path $ExtractPath "ninja.exe"

Write-Host "Starting Ninja build tool installation..." -ForegroundColor Green
Write-Host "Version: $Version" -ForegroundColor Yellow
Write-Host "Expected SHA256: $Sha256" -ForegroundColor Yellow
Write-Host "Download URL: $downloadUrl" -ForegroundColor Yellow
Write-Host "Install Path: $ExtractPath" -ForegroundColor Yellow

try {
    # Step 1: Download the file
    Write-Host "`nDownloading Ninja build tool..." -ForegroundColor Cyan
    
    # Remove existing download if it exists
    if (Test-Path $downloadPath) {
        Remove-Item $downloadPath -Force
        Write-Host "Removed existing download file." -ForegroundColor Yellow
    }
    
    # Download with progress
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $downloadPath -UseBasicParsing -ErrorAction Stop
        Write-Host "Download request completed." -ForegroundColor Yellow
    } catch {
        throw "Download failed: $($_.Exception.Message)"
    }
    
    if (-not (Test-Path $downloadPath)) {
        throw "Download failed: File not found at $downloadPath after download completed"
    }
    
    $fileSize = (Get-Item $downloadPath).Length
    Write-Host "Download completed. File size: $([math]::Round($fileSize / 1KB, 2)) KB" -ForegroundColor Green

    # Step 2: Validate SHA256 hash
    Write-Host "`nValidating SHA256 hash..." -ForegroundColor Cyan
    
    $actualHash = (Get-FileHash -Path $downloadPath -Algorithm SHA256).Hash.ToLower()
    $expectedHash = $Sha256.ToLower()
    
    Write-Host "Expected: $expectedHash" -ForegroundColor Yellow
    Write-Host "Actual:   $actualHash" -ForegroundColor Yellow
    
    if ($actualHash -ne $expectedHash) {
        throw "SHA256 validation failed!`nExpected: $expectedHash`nActual: $actualHash`nThe downloaded file may be corrupted or tampered with."
    }
    
    Write-Host "SHA256 validation passed!" -ForegroundColor Green

    # Step 3: Extract the archive
    Write-Host "`nExtracting archive..." -ForegroundColor Cyan
    
    # Create extraction directory if it doesn't exist
    if (Test-Path $ExtractPath) {
        Write-Host "Removing existing Ninja directory..." -ForegroundColor Yellow
        Remove-Item $ExtractPath -Recurse -Force
    }
    
    New-Item -ItemType Directory -Path $ExtractPath -Force | Out-Null
    Write-Host "Created directory: $ExtractPath" -ForegroundColor Yellow
    
    # Extract using Expand-Archive (PowerShell Core compatible)
    Expand-Archive -Path $downloadPath -DestinationPath $ExtractPath -Force
    
    # Verify ninja exists
    if (-not (Test-Path $ninjaExePath)) {
        throw "Extraction failed: ninja not found at $ninjaExePath"
    }
    
    Write-Host "Extraction completed successfully!" -ForegroundColor Green

    Write-Host "`nAdding to machine PATH..." -ForegroundColor Cyan
    
    # Get current machine PATH
    $machinePathKey = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
    $currentPath = (Get-ItemProperty -Path $machinePathKey -Name PATH).PATH
    
    # Check if path already exists
    $pathEntries = $currentPath -split ';'
    if ($pathEntries -contains $ExtractPath) {
        Write-Host "Path already exists in machine PATH: $ExtractPath" -ForegroundColor Yellow
    } else {
        # Add new path
        $newPath = "$currentPath;$ExtractPath"
        Write-Host "##vso[task.setvariable variable=PATH;]$newPath"
        Write-Host "Added to machine PATH: $ExtractPath" -ForegroundColor Green
    }

    # Cleanup
    Write-Host "`nCleaning up..." -ForegroundColor Cyan
    Remove-Item $downloadPath -Force
    Write-Host "Removed temporary download file." -ForegroundColor Yellow
} catch {
    Write-Host "`n❌ Installation failed!" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    
    # Cleanup on error
    if (Test-Path $downloadPath) {
        Remove-Item $downloadPath -Force -ErrorAction SilentlyContinue
        Write-Host "Cleaned up temporary download file." -ForegroundColor Yellow
    }
    
    # Re-throw the error to maintain error exit code
    throw
}
