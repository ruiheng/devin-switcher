# Release-build and copy installers to .\dist\
# Default: NSIS only. -WithMsi also builds the WiX MSI (the release
# compile dominates; MSI's candle/light adds ~half a minute).
param([switch]$WithMsi)
$ErrorActionPreference = "Stop"
$root = $PSScriptRoot
Push-Location "$root\app"
try {
    if ($WithMsi) { npx --yes @tauri-apps/cli@2 build }
    else { npx --yes @tauri-apps/cli@2 build --bundles nsis }
} finally {
    Pop-Location
}
$dist = "$root\dist"
New-Item -ItemType Directory -Force $dist | Out-Null
Copy-Item "$root\app\target\release\bundle\nsis\*.exe"  $dist -Force -ErrorAction SilentlyContinue
Copy-Item "$root\app\target\release\bundle\msi\*.msi"   $dist -Force -ErrorAction SilentlyContinue
Get-ChildItem $dist | Select-Object Name, Length
