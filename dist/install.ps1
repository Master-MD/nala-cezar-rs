# NALA-Cezar-RS v0.21.0-nala.6.6.6 — Cryptographically Verified Windows x64 Installer
$ErrorActionPreference = "Stop"

$Repo = "Master-MD/nala-cezar-rs"
$Version = if ($env:NALA_CEZAR_VERSION) { $env:NALA_CEZAR_VERSION } else { "v0.21.0-nala.6.6.6" }
$InstallDir = if ($env:NALA_INSTALL_DIR) { $env:NALA_INSTALL_DIR } else { Join-Path $HOME ".local\bin" }
$Asset = "nala-cezar-rs-windows-x86_64.exe"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

try {
    $BaseUrl = "https://github.com/$Repo/releases/download/$Version"
    Write-Host "[NALA-Cezar-RS] Downloading $Asset ($Version)..."
    Invoke-WebRequest -Uri "$BaseUrl/$Asset" -OutFile (Join-Path $TmpDir $Asset)
    Invoke-WebRequest -Uri "$BaseUrl/SHA256SUMS.txt" -OutFile (Join-Path $TmpDir "SHA256SUMS.txt")

    $Sums = Get-Content (Join-Path $TmpDir "SHA256SUMS.txt")
    $Line = $Sums | Where-Object { $_ -match "  $Asset$" } | Select-Object -First 1
    if (-not $Line) { throw "Asset entry missing in SHA256SUMS.txt" }
    $Expected = ($Line -split "\s+")[0].ToLower()
    $Actual = (Get-FileHash -Algorithm SHA256 (Join-Path $TmpDir $Asset)).Hash.ToLower()

    if ($Expected -ne $Actual) {
        throw "SHA-256 mismatch! Expected $Expected, got $Actual"
    }

    $Dest = Join-Path $InstallDir "nala-cezar-rs.exe"
    $Alias = Join-Path $InstallDir "cez-rs.exe"
    Copy-Item -Force (Join-Path $TmpDir $Asset) $Dest
    Copy-Item -Force (Join-Path $TmpDir $Asset) $Alias
    Write-Host "[OK] Installed to $Dest (alias: cez-rs.exe)"
    & $Dest --version
} finally {
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}
