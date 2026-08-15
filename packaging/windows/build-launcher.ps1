$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$outputDir = Join-Path $repoRoot "target\release"
$outputPath = Join-Path $outputDir "waylyrics-launcher.exe"

New-Item -ItemType Directory -Path $outputDir -Force | Out-Null

$rustup = Join-Path $env:USERPROFILE ".cargo\bin\rustup.exe"
if (-not (Test-Path -LiteralPath $rustup)) {
    throw "rustup.exe was not found. Install Rust before building the launcher."
}

$sourcePath = Join-Path $PSScriptRoot "launcher.rs"
$remapFrom = $env:USERPROFILE

& $rustup run stable-x86_64-pc-windows-gnu rustc `
    $sourcePath `
    --edition 2021 `
    -O `
    -C panic=abort `
    -C strip=symbols `
    -C link-arg=-mwindows `
    --remap-path-prefix "$remapFrom=C:\build" `
    -o $outputPath

if ($LASTEXITCODE -ne 0) {
    throw "Failed to build the Windows launcher."
}

Write-Output $outputPath
