# Ozel kisayol: ayarlarda Ctrl+Shift+K iken yalniz o kisayol popup acar, eski Ctrl+Alt+M acmaz.
. "$PSScriptRoot\setup.ps1"
$ErrorActionPreference = "Stop"
$results = [ordered]@{}
function Record($id, $ok, $note) { $results[$id] = "$(if ($ok -eq $null) { 'NOT_VERIFIED' } elseif ($ok) { 'PASS' } else { 'FAIL' }) - $note"; Write-Host "$id : $($results[$id])" }
function Vis { @(Get-AppWindows $script:app.Id $true) }
function Send-Custom { Key 0x11 $true; Key 0x10 $true; Start-Sleep -Milliseconds 40; Key 0x4B $true; Start-Sleep -Milliseconds 60; Key 0x4B $false; Key 0x10 $false; Key 0x11 $false }

Close-Explorers "kd-e2e"; Stop-App; New-Fixture; Write-TestSettings @{ shortcut = "Ctrl+Shift+K" }
$script:app = Start-App
try {
  $ex = Open-Explorer (Join-Path $script:Base "files"); Start-Sleep -Milliseconds 800
  Select-Items $ex @("a.txt"); Focus-Hwnd $ex.HWND; Move-Mouse 700 420
  Send-Hotkey; Start-Sleep -Milliseconds 1000
  Record "SC-old" (@(Vis).Count -eq 0) "eski Ctrl+Alt+M: pencere $(@(Vis).Count)"
  Send-Custom; Start-Sleep -Milliseconds 1100
  Record "SC-new" (@(Vis).Count -eq 1) "yeni Ctrl+Shift+K: pencere $(@(Vis).Count)"
  Press 0x1B; Start-Sleep -Milliseconds 500
} finally { Stop-App; Close-Explorers "kd-e2e"; Restore-Settings }
Write-Host "`n=== OZET ==="
$results.GetEnumerator() | ForEach-Object { "$($_.Key) : $($_.Value)" }
