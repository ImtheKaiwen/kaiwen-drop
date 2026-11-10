# Yerlesim dogrulamasi: farkli imlec konumlarinda popup dikdortgenleri calisma alani icinde mi, ust uste biniyor mu?
. "$PSScriptRoot\setup.ps1"
$ErrorActionPreference = "Stop"
$results = @()
function Record($id, $ok, $note) { $script:results += "$id : $(if ($ok) { 'PASS' } else { 'FAIL' }) - $note"; Write-Host $script:results[-1] }
Add-Type -AssemblyName System.Windows.Forms
$wa = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
Write-Host "WorkingArea: $($wa.Left),$($wa.Top),$($wa.Right),$($wa.Bottom)"

New-Fixture; Write-TestSettings
$app = Start-App
try {
  $ex = Open-Explorer (Join-Path $script:Base "files"); Start-Sleep -Milliseconds 800
  foreach ($case in @(@("orta", 600, 300), @("sag-alt", 1850, 980), @("sol-ust", 20, 20))) {
    Focus-Hwnd $ex.HWND
    Select-Items $ex @("a.txt", "b.txt"); Start-Sleep -Milliseconds 300; Focus-Hwnd $ex.HWND
    Move-Mouse $case[1] $case[2]; Start-Sleep -Milliseconds 200
    Send-Hotkey; Start-Sleep -Milliseconds 1000
    $root = @(Get-AppWindows $app.Id $true) | Select-Object -First 1
    if (-not $root) { Record "geo-$($case[0])" $false "kok popup yok"; continue }
    $s = $root.Dpi / 96.0; $m = [int][Math]::Round(10 * $s, [MidpointRounding]::AwayFromZero)
    # Documents satiri
    Move-Mouse ([int](($root.L + $root.R) / 2)) ([int]($root.T + (10 + 1 + 6 + 34 + 17) * $s)); Start-Sleep -Milliseconds 700
    $w = @(Get-AppWindows $app.Id $true)
    $line = ($w | ForEach-Object { "[$($_.L + $m),$($_.T + $m),$($_.R - $m),$($_.B - $m)]" }) -join " "
    Write-Host "$($case[0]): $line"
    $inside = $true
    foreach ($x in $w) { if (($x.L + $m) -lt $wa.Left -or ($x.T + $m) -lt $wa.Top -or ($x.R - $m) -gt $wa.Right -or ($x.B - $m) -gt $wa.Bottom) { $inside = $false } }
    Record "geo-$($case[0])" ($inside -and $w.Count -eq 2) "$($w.Count) popup, calisma alani icinde=$inside"
    [void](Save-Shot "geo-$($case[0])")
    # 3. seviye: Documents popup'inda Together (action, sep, Personal, Partner, Together)
    $c = @($w | Sort-Object { $_.R - $_.L } -Descending) | Where-Object { $_.Hwnd -ne $root.Hwnd } | Select-Object -First 1
    Move-Mouse ([int](($c.L + $c.R) / 2)) ([int]($c.T + (10 + 1 + 6 + 34 + 9 + 3 * 34 + 17) * $s)); Start-Sleep -Milliseconds 800
    $w3 = @(Get-AppWindows $app.Id $true)
    Write-Host "$($case[0]) L3: $(($w3 | ForEach-Object { "[$($_.L + $m),$($_.T + $m),$($_.R - $m),$($_.B - $m)]" }) -join ' ')"
    $ov = $false
    for ($i = 0; $i -lt $w3.Count; $i++) { for ($j = $i + 1; $j -lt $w3.Count; $j++) {
      $a = $w3[$i]; $b = $w3[$j]
      if (($a.L + $m) -lt ($b.R - $m) -and ($b.L + $m) -lt ($a.R - $m) -and ($a.T + $m) -lt ($b.B - $m) -and ($b.T + $m) -lt ($a.B - $m)) { $ov = $true } } }
    Record "geo3-$($case[0])" ($w3.Count -eq 3 -and -not $ov) "$($w3.Count) popup, ust uste binme=$ov"
    [void](Save-Shot "geo3-$($case[0])")
    Press 0x1B; Start-Sleep -Milliseconds 500
  }
} finally { Stop-App; Close-Explorers "kd-e2e" }
$results
