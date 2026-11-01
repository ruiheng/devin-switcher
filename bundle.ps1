# Release-build and copy installers to .\dist\
# -NsisOnly skips the WiX MSI (smaller win than it looks: the release
# compile dominates; MSI's candle/light is only ~half a minute).
param([switch]$NsisOnly)
$ErrorActionPreference = "Stop"
$root = $PSScriptRoot
Push-Location "$root\app"
try {
    if ($NsisOnly) { npx --yes @tauri-apps/cli@2 build --bundles nsis }
    else { npx --yes @tauri-apps/cli@2 build }
} finally {
    Pop-Location
}
$dist = "$root\dist"
New-Item -ItemType Directory -Force $dist | Out-Null
Copy-Item "$root\app\target\release\bundle\nsis\*.exe"  $dist -Force -ErrorAction SilentlyContinue
Copy-Item "$root\app\target\release\bundle\msi\*.msi"   $dist -Force -ErrorAction SilentlyContinue
Get-ChildItem $dist | Select-Object Name, Length
