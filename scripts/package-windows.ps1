$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo

$metadata = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
$version = ($metadata.packages | Where-Object name -eq "kfnotepad").version
if (-not $version) {
    throw "Could not determine package version from Cargo.toml."
}

$dist = if ($env:KFNOTEPAD_DIST_DIR) { $env:KFNOTEPAD_DIST_DIR } else { "dist" }
$target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { "target" }
$name = "kfnotepad-$version-windows-x86_64"
$stage = Join-Path $target "package/$name"

cargo build --locked --release --no-default-features --features tui --bin kfnotepad
cargo build --locked --release --no-default-features --features gui --bin kfnotepad-gui

Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
New-Item (Join-Path $stage "docs") -ItemType Directory -Force | Out-Null
New-Item $dist -ItemType Directory -Force | Out-Null

Copy-Item (Join-Path $target "release/kfnotepad.exe") $stage
Copy-Item (Join-Path $target "release/kfnotepad-gui.exe") $stage
Copy-Item "LICENSE", "README.md" $stage
Copy-Item "docs/13-OPERATIONS.md", "docs/16-CLI-CONTRACT.md", "docs/17-GUI-CONTRACT.md" (Join-Path $stage "docs")

$zip = Join-Path $dist "$name.zip"
Remove-Item $zip -Force -ErrorAction SilentlyContinue
Compress-Archive -Path $stage -DestinationPath $zip -CompressionLevel Optimal
Copy-Item (Join-Path $target "release/kfnotepad.exe") (Join-Path $dist "kfnotepad-$version-windows-x86_64.exe")
Copy-Item (Join-Path $target "release/kfnotepad-gui.exe") (Join-Path $dist "kfnotepad-gui-$version-windows-x86_64.exe")

Get-ChildItem $dist -File | ForEach-Object {
    $hash = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $($_.Name)" | Set-Content "$($_.FullName).sha256" -Encoding ascii
}

Write-Host "Created Windows release artifacts in $dist"
