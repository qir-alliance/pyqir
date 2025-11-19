# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

# Print current PATH
Write-BuildLog "=== CURRENT PATH ==="

# Use platform-appropriate path separator
$pathSeparator = if ($IsWindows) { ';' } else { ':' }
$env:PATH -split $pathSeparator | ForEach-Object { Write-BuildLog $_ }

Write-BuildLog "`n=== ALL ENVIRONMENT VARIABLES ==="
Get-ChildItem Env: | Sort-Object Name | Format-Table Name, Value -AutoSize

Write-BuildLog "`n=== EXECUTABLE APPLICATIONS ON PATH ==="

# Get all directories in PATH
$pathDirs = $env:PATH -split $pathSeparator | Where-Object { $_ -and (Test-Path $_ -ErrorAction SilentlyContinue) }

# Define executable extensions based on platform
$executableExtensions = if ($IsWindows) {
    @('.exe', '.cmd', '.bat', '.ps1', '.com', '.msi', '.vbs', '.js', '.jar')
} else {
    @('') # On Unix-like systems, executables often have no extension
}

# Collect all executables
$executables = @()

foreach ($dir in $pathDirs) {
    try {
        Get-ChildItem -Path $dir -File -ErrorAction SilentlyContinue | Where-Object {
            if ($IsWindows) {
                $executableExtensions -contains $_.Extension
            } else {
                # On Unix-like systems, check if file is executable
                (Test-Path $_.FullName -PathType Leaf) -and 
                ((Get-Item $_.FullName).UnixMode -match '^.{0,2}x' -or $_.Extension -eq '' -or $executableExtensions -contains $_.Extension)
            }
        } | ForEach-Object {
            $executables += [PSCustomObject]@{
                FileName = $_.Name
                FullPath = $_.FullName
            }
        }
    }
    catch {
        Write-BuildLog "Warning: Could not access directory $dir. Skipping."
        # Skip directories that can't be accessed
        continue
    }
}

# Remove duplicates (keep first occurrence) and sort by filename
$uniqueExecutables = $executables | Sort-Object FileName | Group-Object FileName | ForEach-Object { $_.Group[0] }

# Display in table format
$uniqueExecutables | Format-Table FileName, FullPath -AutoSize

Write-BuildLog "`nTotal executable applications found: $($uniqueExecutables.Count)"
Write-BuildLog "Platform: $(if ($IsWindows) { 'Windows' } elseif ($IsLinux) { 'Linux' } elseif ($IsMacOS) { 'macOS' } else { 'Unknown' })"