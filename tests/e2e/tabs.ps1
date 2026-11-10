# Sekmeli Explorer: yalniz AKTIF sekmenin secimi tasinmali (arka plandaki sekmenin secimi degil).
. "$PSScriptRoot\setup.ps1"
$ErrorActionPreference = "Stop"
$results = [ordered]@{}
function Record($id, $ok, $note) { $results[$id] = "$(if ($ok -eq $null) { 'NOT_VERIFIED' } elseif ($ok) { 'PASS' } else { 'FAIL' }) - $note"; Write-Host "$id : $($results[$id])" }
function Vis { @(Get-AppWindows $script:app.Id $true) }
function RowY($w, [int]$i, [bool]$after = $false) { $s = $w.Dpi / 96.0; [int]($w.T + (10 + 1 + 6 + $i * 34 + $(if ($after -and $i -ge 1) { 9 } else { 0 }) + 17) * $s) }

Close-Explorers "kd-e2e"; Stop-App; New-Fixture; Write-TestSettings @{}
$script:app = Start-App
try {
  $shell = New-Object -ComObject Shell.Application
  $tab1 = Open-Explorer (Join-Path $script:Base "files"); Start-Sleep -Milliseconds 800
  $hwnd = $tab1.HWND
  Select-Items $tab1 @("a.txt"); Focus-Hwnd $hwnd
  [W]::keybd_event(0x11, 0, 0, [UIntPtr]::Zero); Press 0x54; [W]::keybd_event(0x11, 0, 2, [UIntPtr]::Zero)   # Ctrl+T: yeni sekme
  Start-Sleep -Milliseconds 1800
  $tabs = @($shell.Windows() | Where-Object { $_.HWND -eq $hwnd })
  $tab2 = $tabs | Where-Object { $_.LocationURL -notlike "*kd-e2e/files*" -and $_.LocationURL -notlike "*kd-e2e%2Ffiles*" } | Select-Object -First 1
  if (-not $tab2 -or $tabs.Count -lt 2) { Record "TABS" $null "ikinci sekme olusturulamadi (sekme sayisi: $($tabs.Count))" }
  else {
    $tab2.Navigate2((Join-Path $script:Base "other")); Start-Sleep -Milliseconds 1500
    Select-Items $tab2 @("x.txt"); Start-Sleep -Milliseconds 500; Focus-Hwnd $hwnd
    Move-Mouse 700 420; Send-Hotkey; Start-Sleep -Milliseconds 1100
    $r = @(Vis)[0]
    if (-not $r) { Record "TABS" $false "popup acilmadi" }
    else {
      $before = @(Vis | ForEach-Object { $_.Hwnd })
      Move-Mouse ([int](($r.L + $r.R) / 2)) (RowY $r 1); Start-Sleep -Milliseconds 800
      $c = @(Vis | Where-Object { $before -notcontains $_.Hwnd })[0]
      Click-At ([int](($c.L + $c.R) / 2)) (RowY $c 2 $true); Start-Sleep -Milliseconds 900     # Partner: tikla = tasi
      Press 0x0D; Start-Sleep -Milliseconds 1300
      $ok = (Test-Path (Join-Path $script:Base "roots\Docs\Partner\x.txt")) -and (Test-Path (Join-Path $script:Base "files\a.txt")) -and -not (Test-Path (Join-Path $script:Base "roots\Docs\Partner\a.txt"))
      $st = "x.txt hedefte=$(Test-Path (Join-Path $script:Base 'roots\Docs\Partner\x.txt')) other\x.txt yerinde=$(Test-Path (Join-Path $script:Base 'other\x.txt')) a.txt hedefte=$(Test-Path (Join-Path $script:Base 'roots\Docs\Partner\a.txt')) files\a.txt yerinde=$(Test-Path (Join-Path $script:Base 'files\a.txt')) popup=$((Vis).Count)"
      Record "TABS" $ok "aktif sekmenin secimi (other\x.txt) tasindi, arka sekmeninki (files\a.txt) yerinde: $st"
    }
  }
} finally { Stop-App; Close-Explorers "kd-e2e"; Restore-Settings }
Write-Host "`n=== OZET ==="
$results.GetEnumerator() | ForEach-Object { "$($_.Key) : $($_.Value)" }
