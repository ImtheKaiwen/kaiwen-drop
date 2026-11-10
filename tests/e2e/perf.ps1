# Performans olcumu (RELEASE exe): bosta RAM (WebView2 alt surecleri DAHIL), bosta CPU, kisayol -> kok popup gecikmesi.
. "$PSScriptRoot\setup.ps1"
$ErrorActionPreference = "Stop"
$script:Exe = (Resolve-Path (Join-Path $PSScriptRoot "..\..\src-tauri\target\release\kaiwen-drop.exe")).Path
function Tree-Procs([int]$RootPid) {
  $all = Get-CimInstance Win32_Process
  $ids = New-Object System.Collections.Generic.HashSet[int]; [void]$ids.Add($RootPid)
  $grew = $true
  while ($grew) { $grew = $false; foreach ($p in $all) { if ($ids.Contains([int]$p.ParentProcessId) -and $ids.Add([int]$p.ProcessId)) { $grew = $true } } }
  $ids | ForEach-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue }
}
function Mem-MB($procs) { [math]::Round((($procs | Measure-Object WorkingSet64 -Sum).Sum) / 1MB, 1) }
function Priv-MB($procs) { [math]::Round((($procs | Measure-Object PrivateMemorySize64 -Sum).Sum) / 1MB, 1) }

Close-Explorers "kd-e2e"; Stop-App; New-Fixture; Write-TestSettings @{}
$app = Start-Process $script:Exe -PassThru
try {
  Start-Sleep -Seconds 6
  $procs = @(Tree-Procs $app.Id)
  "Bosta (6 sn sonra): surec sayisi=$($procs.Count)  toplam WorkingSet=$(Mem-MB $procs) MB  ozel bellek=$(Priv-MB $procs) MB"
  "  ana exe WorkingSet: $(Mem-MB @(Get-Process -Id $app.Id)) MB; WebView2 cocuklari: $(Mem-MB @($procs | Where-Object { $_.Id -ne $app.Id })) MB"
  $c0 = ($procs | Measure-Object TotalProcessorTime -Sum -ErrorAction SilentlyContinue); $t0 = ($procs | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum
  Start-Sleep -Seconds 20
  $procs2 = @(Tree-Procs $app.Id)
  $t1 = ($procs2 | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum
  "app exited after idle? $($app.HasExited)"
  $cores = [Environment]::ProcessorCount
  "Bosta CPU (20 sn): $([math]::Round($t1 - $t0, 3)) cpu-sn => ortalama %$([math]::Round(($t1 - $t0) / 20 / $cores * 100, 3)) (toplam $cores mantiksal cekirdek uzerinden)"

  function Press-Measure($ex) {
    Focus-Hwnd $ex.HWND; Start-Sleep -Milliseconds 300
    $sw = [Diagnostics.Stopwatch]::StartNew(); Send-Hotkey; $seen = $false
    while ($sw.ElapsedMilliseconds -lt 4000) { if (@(Get-AppWindows $app.Id $true).Count -ge 1) { $seen = $true; break }; Start-Sleep -Milliseconds 5 }
    $ms = $sw.ElapsedMilliseconds - 60   # Send-Hotkey tusu 60 ms basili tutar
    Start-Sleep -Milliseconds 500; Press 0x1B; Start-Sleep -Milliseconds 800
    if ($seen) { $ms } else { -1 }
  }
  $ex = Open-Explorer (Join-Path $script:Base "files"); Start-Sleep -Milliseconds 800
  Focus-Hwnd $ex.HWND; Select-Items $ex @("a.txt", "b.txt"); Start-Sleep -Milliseconds 300; Focus-Hwnd $ex.HWND
  Move-Mouse 700 420
  $cold1 = Press-Measure $ex
  "Ilk kisayol (SOGUK: WebView henuz yok) -> gorunur kok popup: $cold1 ms"
  $warm = @(1..6 | ForEach-Object { Press-Measure $ex })
  "Sonraki kisayollar (SICAK) n=$($warm.Count): $($warm -join ', ') ms; medyan=$(($warm | Sort-Object)[[int]($warm.Count / 2)]) ms"
  $p3 = @(Tree-Procs $app.Id); "Kullanimdan hemen sonra: surec=$($p3.Count) WorkingSet=$(Mem-MB $p3) MB ozel=$(Priv-MB $p3) MB"
  Focus-Hwnd $ex.HWND; Send-Hotkey; Start-Sleep -Milliseconds 900
  $r = @(Get-AppWindows $app.Id $true)[0]; $s = $r.Dpi / 96.0
  $sw = [Diagnostics.Stopwatch]::StartNew()
  Move-Mouse ([int](($r.L + $r.R) / 2)) ([int]($r.T + (10 + 1 + 6 + 34 + 17) * $s))
  while ($sw.ElapsedMilliseconds -lt 3000) { if (@(Get-AppWindows $app.Id $true).Count -ge 2) { break }; Start-Sleep -Milliseconds 5 }
  "Hover -> alt menu gorunur: $($sw.ElapsedMilliseconds) ms (130 ms niyet gecikmesi dahil)"
  Press 0x1B; Start-Sleep -Milliseconds 800
  "Bosalmayi bekliyor (150 sn)..."; Start-Sleep -Seconds 150
  $p4 = @(Tree-Procs $app.Id); "150 sn bosta kaldiktan sonra: surec=$($p4.Count) WorkingSet=$(Mem-MB $p4) MB ozel=$(Priv-MB $p4) MB; Tauri pencere sayisi=$(@(Get-AppWindows $app.Id $false).Count)"
  $cold2 = Press-Measure $ex
  "Bosalmadan sonra ilk kisayol (SOGUK) -> kok popup: $cold2 ms"} finally { Stop-App; Close-Explorers "kd-e2e"; Restore-Settings }
