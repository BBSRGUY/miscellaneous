# Package Forge release binaries for Windows distribution
param(
    [string]$Version = "",
    [string]$Target = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"

# Get version from Cargo.toml if not provided
if ([string]::IsNullOrEmpty($Version)) {
    $cargoToml = Get-Content "Cargo.toml" -Raw
    if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
        $Version = $matches[1]
    } else {
        Write-Error "Could not determine version"
        exit 1
    }
}

$OutputDir = "release-packages"
$Platform = "windows"
$Arch = "x64"
$PackageName = "forge-v$Version-$Platform-$Arch"
$PackageDir = "$OutputDir\$PackageName"

Write-Host "Packaging Forge v$Version for Windows" -ForegroundColor Green

# Build release binaries
Write-Host "Building release binaries..." -ForegroundColor Yellow
cargo build --release --workspace

# Create output directory
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
New-Item -ItemType Directory -Force -Path "$PackageDir\bin" | Out-Null
New-Item -ItemType Directory -Force -Path "$PackageDir\configs" | Out-Null
New-Item -ItemType Directory -Force -Path "$PackageDir\docs" | Out-Null

Write-Host "Creating package: $PackageName" -ForegroundColor Yellow

# Copy binaries
Write-Host "Copying binaries..." -ForegroundColor Yellow
Copy-Item "target\release\forge.exe" "$PackageDir\bin\"

# Copy configuration files
Write-Host "Copying configuration files..." -ForegroundColor Yellow
Copy-Item "configs\default.yaml" "$PackageDir\configs\"
if (Test-Path "configs\local.yaml.example") {
    Copy-Item "configs\local.yaml.example" "$PackageDir\configs\"
}

# Copy documentation
Write-Host "Copying documentation..." -ForegroundColor Yellow
Copy-Item "README.md" "$PackageDir\"
if (Test-Path "LICENSE") {
    Copy-Item "LICENSE" "$PackageDir\"
}

# Create installation instructions
$installText = @"
Forge - Local LLM Platform
Installation Instructions for Windows

1. Extract this archive to your desired installation directory
2. Add the bin\ directory to your PATH:
   - Open System Properties > Environment Variables
   - Edit the PATH variable
   - Add the full path to the bin\ directory

3. Run the daemon:
   forge.exe serve

4. Use the CLI:
   forge.exe chat
   forge.exe models list
   forge.exe --help

Configuration:
- Default configuration is in configs\default.yaml
- Create a local.yaml for custom settings

For more information, see README.md
"@

Set-Content -Path "$PackageDir\INSTALL.txt" -Value $installText

# Create ZIP archive
Write-Host "Creating archive..." -ForegroundColor Yellow
$zipPath = "$OutputDir\$PackageName.zip"
if (Test-Path $zipPath) {
    Remove-Item $zipPath
}

Compress-Archive -Path $PackageDir -DestinationPath $zipPath

Write-Host "Created: $zipPath" -ForegroundColor Green

# Generate checksum
Write-Host "Generating checksum..." -ForegroundColor Yellow
$hash = Get-FileHash -Path $zipPath -Algorithm SHA256
Set-Content -Path "$zipPath.sha256" -Value "$($hash.Hash.ToLower())  $PackageName.zip"

Write-Host ""
Write-Host "Packaging complete!" -ForegroundColor Green
Write-Host "Package location: $OutputDir\" -ForegroundColor Green
Get-ChildItem $OutputDir | Format-Table Name, Length, LastWriteTime
