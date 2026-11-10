# Kose dock'u: sag-alt kosede, calisma alani icinde; birakma alani acilinca dock'un ustunde durur.
. "$PSScriptRoot\setup.ps1"
$ErrorActionPreference = "Stop"
function Vis { @(Get-AppWindows $script:app.Id $true) }
Close-Explorers "kd-e2e"; Stop-App; New-Fixture; Write-TestSettings @{ corner_dock = $true }
$script:app = Start-App
try {
  Start-Sleep -Milliseconds 2500
  $d = @(Vis)
  $wa = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
  $dock = $d | Select-Object -First 1
  if (-not $dock) { Write-Host "DOCK : FAIL - dock penceresi yok"; return }
  $s = $dock.Dpi / 96.0
  $inCorner = ($dock.R -le ($wa.Right)) -and ($dock.B -le ($wa.Bottom)) -and (($wa.Right - $dock.R) -lt 40 * $s) -and (($wa.Bottom - $dock.B) -lt 40 * $s)
  Write-Host "DOCK : $(if ($inCorner) { 'PASS' } else { 'FAIL' }) - dock [$($dock.L),$($dock.T),$($dock.R),$($dock.B)] calisma alani sag-alt [$($wa.Right),$($wa.Bottom)]"
  Start-Process $script:Exe -ArgumentList "--drop" | Out-Null; Start-Sleep -Milliseconds 2500
  $all = @(Vis); $surf = $all | Where-Object { $_.Hwnd -ne $dock.Hwnd } | Select-Object -First 1
  $ok = $surf -and ($surf.B -le $dock.T) -and ($surf.R -le $wa.Right)
  Write-Host "DOCK-SURFACE : $(if ($ok) { 'PASS' } else { 'FAIL' }) - birakma alani dock'un ustunde ve ortusmuyor ($($all.Count) pencere)"
  $l = ($all | Measure-Object L -Minimum).Minimum; $t = ($all | Measure-Object T -Minimum).Minimum; $r = ($all | Measure-Object R -Maximum).Maximum; $b = ($all | Measure-Object B -Maximum).Maximum
  [void](Save-Crop "19-dock-corner" ($l - 80) ($t - 80) (($r - $l) + 160) (($b - $t) + 160))
} finally { Stop-App; Restore-Settings }
