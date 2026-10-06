# Release-build and copy installers to .\dist\
$ErrorActionPreference = "Stop"
$root = $PSScriptRoot
Push-Location "$root\app"
try {
    npx --yes @tauri-apps/cli@2 build
} finally {
    Pop-Location
}
$dist = "$root\dist"
New-Item -ItemType Directory -Force $dist | Out-Null
Copy-Item "$root\app\target\release\bundle\nsis\*.exe"  $dist -Force -ErrorAction SilentlyContinue
Copy-Item "$root\app\target\release\bundle\msi\*.msi"   $dist -Force -ErrorAction SilentlyContinue
Get-ChildItem $dist | Select-Object Name, Length
