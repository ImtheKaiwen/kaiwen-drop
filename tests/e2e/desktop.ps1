# W05 - Masaustunde coklu secim (gercek masaustu simgeleri, Ctrl+tikla) ve masaustunde bos secim.
# Yalniz secim yapar; masaustundeki hicbir oge degistirilmez/tasinmaz. Win+D ile on plana getirir ve geri alir.
. "$PSScriptRoot\setup.ps1"
$ErrorActionPreference = "Stop"
$results = [ordered]@{}
function Record($id, $ok, $note) { $results[$id] = "$(if ($ok -eq $null) { 'NOT_VERIFIED' } elseif ($ok) { 'PASS' } else { 'FAIL' }) - $note"; Write-Host "$id : $($results[$id])" }
function Vis { @(Get-AppWindows $script:app.Id $true) }

Close-Explorers "kd-e2e"; Stop-App; New-Fixture; Write-TestSettings @{}
$script:app = Start-App
$sh = New-Object -ComObject Shell.Application
try {
  $sh.ToggleDesktop(); Start-Sleep -Milliseconds 1500   # Win+D: masaustu on plana gelir (test sonunda geri alinir)
  Click-At 46 40; Start-Sleep -Milliseconds 400          # ilk masaustu simgesi
  [W]::keybd_event(0x11, 0, 0, [UIntPtr]::Zero); Click-At 46 145; [W]::keybd_event(0x11, 0, 2, [UIntPtr]::Zero)   # Ctrl+tikla: ikinci simge
  Start-Sleep -Milliseconds 600
  Move-Mouse 700 420
  $fg = [W]::GetForegroundWindow(); $cn = New-Object Text.StringBuilder 64; [void][W]::GetClassName($fg, $cn, 64)
  Send-Hotkey; Start-Sleep -Milliseconds 1100
  Record "W05" (@(Vis).Count -eq 1) "masaustunde 2 simge secili (on plan: $cn) + kisayol: menu penceresi $(@(Vis).Count)"
  [void](Save-Shot "15-desktop-selection-popup")
  Press 0x1B; Start-Sleep -Milliseconds 600
  Click-At 1000 300; Start-Sleep -Milliseconds 600       # bos masaustu alani: secim kalkar
  Send-Hotkey; Start-Sleep -Milliseconds 900
  Record "W05b" (@(Vis).Count -eq 0) "masaustunde secim yok + kisayol: pencere $(@(Vis).Count)"
  [void](Save-Shot "16-desktop-empty-selection")
  $sh.ToggleDesktop(); Start-Sleep -Milliseconds 1200   # pencereleri geri getir
} finally { Stop-App; Close-Explorers "kd-e2e"; Restore-Settings }
Write-Host "`n=== OZET ==="
$results.GetEnumerator() | ForEach-Object { "$($_.Key) : $($_.Value)" }
