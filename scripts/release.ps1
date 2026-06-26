# Build release and package into a zip with http/ and lua/ directories
param()

$ErrorActionPreference = "Stop"

$ProjectDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $ProjectDir

# Extract package info from Cargo.toml
$CargoToml = Get-Content "Cargo.toml" -Raw
$PkgName = [regex]::Match($CargoToml, '^name\s*=\s*"([^"]+)"', 'Multiline').Groups[1].Value
$PkgVersion = [regex]::Match($CargoToml, '^version\s*=\s*"([^"]+)"', 'Multiline').Groups[1].Value

Write-Host "Building $PkgName v$PkgVersion in release mode..."
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$TargetDir = Join-Path $ProjectDir "target\release"
$ExeName = "$PkgName.exe"
$ExePath = Join-Path $TargetDir $ExeName

if (-not (Test-Path $ExePath)) {
    Write-Error "Executable not found at: $ExePath"
    exit 1
}

$ZipName = "$PkgName-$PkgVersion.zip"
$ZipPath = Join-Path $TargetDir $ZipName

# Create a staging directory for clean zip structure
$StagingDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $StagingDir | Out-Null

try {
    Copy-Item $ExePath -Destination $StagingDir
    Copy-Item (Join-Path $ProjectDir "http") -Destination $StagingDir -Recurse
    Copy-Item (Join-Path $ProjectDir "lua") -Destination $StagingDir -Recurse

    # Remove old zip if it exists
    if (Test-Path $ZipPath) { Remove-Item $ZipPath }

    Compress-Archive -Path "$StagingDir\*" -DestinationPath $ZipPath -Force
    Write-Host "Release zip created: $ZipPath"
} finally {
    Remove-Item -Recurse -Force $StagingDir
}
