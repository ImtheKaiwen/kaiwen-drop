# Kok klasor yokken Ayarlar gorunumu (bos durum) - ekran goruntusu.
. "$PSScriptRoot\setup.ps1"
$ErrorActionPreference = "Stop"
function Vis { @(Get-AppWindows $script:app.Id $true) }
foreach ($theme in @("light", "dark")) {
  Close-Explorers "kd-e2e"; Stop-App; New-Fixture; Write-TestSettings @{ roots = @(); theme = $theme }
  $script:app = Start-App
  try {
    Start-Process $script:Exe | Out-Null; Start-Sleep -Milliseconds 3000
    $w = @(Vis)[0]
    if (-not $w) { Write-Host "EMPTY-$theme : FAIL - pencere yok"; continue }
    Focus-Hwnd $w.Hwnd
    [void](Save-Crop "18-settings-empty-roots-$theme" ($w.L - 20) ($w.T - 20) (($w.R - $w.L) + 40) (($w.B - $w.T) + 40))
    Write-Host "EMPTY-$theme : PASS - ekran goruntusu alindi"
  } finally { Stop-App; Restore-Settings }
}
