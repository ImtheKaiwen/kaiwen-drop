# Risk spike: bos secim / secim / kaskad popup. Cikti: PASS/FAIL satirlari + ekran goruntuleri.
. "$PSScriptRoot\setup.ps1"
$ErrorActionPreference = "Stop"
$results = @()
function Record($id, $ok, $note) { $script:results += "$id : $(if ($ok) { 'PASS' } else { 'FAIL' }) - $note"; Write-Host $script:results[-1] }

New-Fixture
Write-TestSettings
$app = Start-App
try {
  $files = Join-Path $script:Base "files"
  $ex = Open-Explorer $files
  Start-Sleep -Milliseconds 800
  Focus-Hwnd $ex.HWND

  # W01: secim yok + kisayol => sifir gorunur pencere
  Clear-Selection $ex; Start-Sleep -Milliseconds 300; Focus-Hwnd $ex.HWND
  Send-Hotkey; Start-Sleep -Milliseconds 900
  $vis = @(Get-AppWindows $app.Id $true)
  Record "W01" ($vis.Count -eq 0) "gorunur uygulama penceresi: $($vis.Count)"
  [void](Save-Shot "w01-empty-selection")

  # W03/W04: coklu secim + kisayol => tek kok popup
  Select-Items $ex @("a.txt", "b.txt", "c-türkçe-ğüşiöç.txt"); Start-Sleep -Milliseconds 400
  Focus-Hwnd $ex.HWND
  $cur = Get-Cursor
  Send-Hotkey; Start-Sleep -Milliseconds 1200
  $vis = @(Get-AppWindows $app.Id $true)
  Record "W03" ($vis.Count -eq 1) "gorunur pencere: $($vis.Count) $(($vis | ForEach-Object { "[$($_.L),$($_.T),$($_.R),$($_.B)] dpi=$($_.Dpi)" }) -join ' ')"
  if ($vis.Count -ge 1) {
    $r = $vis[0]; [void](Save-Crop "w03-root-popup" ($r.L - 40) ($r.T - 40) (($r.R - $r.L) + 80) (($r.B - $r.T) + 80))
    $scale = $r.Dpi / 96.0
    # 2. satir (Documents): yuzey top = T+10*scale
    $rowY = [int]($r.T + (10 + 1 + 6 + 34 + 17) * $scale)
    $rowX = [int](($r.L + $r.R) / 2)
    Move-Mouse $rowX $rowY; Start-Sleep -Milliseconds 700
    $vis = @(Get-AppWindows $app.Id $true)
    Record "W06" ($vis.Count -eq 2) "alt menu: $($vis.Count) pencere"
    [void](Save-Shot "w06-cascade-2")
    if ($vis.Count -ge 2) {
      $c = $vis | Sort-Object L | Select-Object -Last 1
      # W07: fareyi alt menuye tasi; zincir kapanmamali
      Move-Mouse ([int](($c.L + $c.R) / 2)) ([int]($c.T + (10 + 1 + 6 + 17) * $scale)); Start-Sleep -Milliseconds 500
      $vis = @(Get-AppWindows $app.Id $true)
      Record "W07" ($vis.Count -eq 2) "alt menuye gecince pencere sayisi: $($vis.Count)"
      # Together = 4. klasor satiri (action, sep, Personal, Partner, Together): index 4 -> 1+...
      $y = [int]($c.T + (10 + 1 + 6 + 34 + 9 + 3 * 34 + 17) * $scale)
      Move-Mouse ([int](($c.L + $c.R) / 2)) $y; Start-Sleep -Milliseconds 800
      $vis = @(Get-AppWindows $app.Id $true)
      Record "W06b" ($vis.Count -eq 3) "ucuncu seviye: $($vis.Count) pencere"
      [void](Save-Shot "w06-cascade-3")
    }
  }
  # W08: Escape
  Press 0x1B; Start-Sleep -Milliseconds 700
  $vis = @(Get-AppWindows $app.Id $true)
  Record "W08" ($vis.Count -eq 0) "Esc sonrasi gorunur pencere: $($vis.Count)"

  # W02: Explorer disi on plan => popup yok
  $np = Start-Process notepad -PassThru; Start-Sleep -Milliseconds 1200
  Send-Hotkey; Start-Sleep -Milliseconds 900
  $vis = @(Get-AppWindows $app.Id $true)
  Record "W02" ($vis.Count -eq 0) "Notepad on planda, popup: $($vis.Count)"
  Stop-Process $np -Force
} finally {
  Stop-App; Close-Explorers "kd-e2e"
}
Write-Host "`n--- OZET ---"; $results
