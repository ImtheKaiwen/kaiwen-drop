# Tum Windows E2E betiklerini sirayla calistirir ve OZETLERI tek dosyada toplar.
# Kullanim: powershell -File tests\e2e\all.ps1   (yaklasik 12 dk; gercek fare/klavye kullanir - bu sirada bilgisayara dokunmayin)
$summary = Join-Path $PSScriptRoot "..\..\docs\e2e-last-run.txt"
$runs = @(
  @("spike.ps1", 150), @("geometry.ps1", 150), @("full.ps1", 420), @("settings.ps1", 150),
  @("desktop.ps1", 90), @("drop.ps1", 120), @("tabs.ps1", 100)
)
$lines = @("Kaiwen Drop E2E - $(Get-Date -Format 'yyyy-MM-dd HH:mm') - Windows $([Environment]::OSVersion.Version) - exe: debug")
foreach ($r in $runs) {
  $out = & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "run.ps1") $r[0] -TimeoutSec $r[1]
  $lines += "", "## $($r[0])"
  $lines += @($out | Where-Object { $_ -match '^(W\d|KBD|SIZE|I18N|ESC|TABS|DROP|DOCK|geo)' } | Select-Object -Unique)
}
foreach ($v in @(@("dark", "tr-TR"), @("light", "en-US"))) {
  $out = & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "run.ps1") settings.ps1 -TimeoutSec 100 -Theme $v[0] -Lang $v[1]
  $lines += "", "## settings.ps1 ($($v[0]) / $($v[1]))"
  $lines += @($out | Where-Object { $_ -match '^(W\d|SIZE|I18N)' } | Select-Object -Unique)
}
[IO.File]::WriteAllLines($summary, $lines, (New-Object Text.UTF8Encoding $false))
$lines
