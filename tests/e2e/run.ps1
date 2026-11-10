# E2E betigini ayri surecte, zaman asimi ile calistirir (takilirsa oldurur, uygulamayi kapatir, ayarlari geri koyar).
# Kullanim: powershell -File tests\e2e\run.ps1 full.ps1 [-TimeoutSec 240] [ek argumanlar...]
param([Parameter(Mandatory)][string]$Script, [int]$TimeoutSec = 240, [Parameter(ValueFromRemainingArguments)]$Rest)
$out = Join-Path $env:TEMP "kd-e2e-$($Script -replace '\W','_').out"
$args2 = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot $Script)) + @($Rest | Where-Object { $_ })
$p = Start-Process powershell -ArgumentList $args2 -RedirectStandardOutput $out -RedirectStandardError "$out.err" -WindowStyle Hidden -PassThru
if (-not $p.WaitForExit($TimeoutSec * 1000)) {
  Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
  Write-Host "ZAMAN ASIMI ($TimeoutSec sn): betik oldurulup uygulama kapatildi"
  . "$PSScriptRoot\setup.ps1"; Stop-App; Restore-Settings
}
Get-Content $out
$e = Get-Content "$out.err" -ErrorAction SilentlyContinue | Select-Object -First 8
if ($e) { Write-Host "--- stderr ---"; $e }
