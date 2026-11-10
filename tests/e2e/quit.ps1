# Cikis E2E: sicak WebView + Ayarlar acikken `--quit` -> tum surecler (WebView2 cocuklari dahil) kapanmali.
. "$PSScriptRoot\setup.ps1"
$ErrorActionPreference = "Stop"
Close-Explorers "kd-e2e"; Stop-App; New-Fixture; Write-TestSettings @{ theme = "light"; language = "tr-TR" }
$script:app = Start-App
Start-Process $script:Exe | Out-Null; Start-Sleep -Milliseconds 3000     # Ayarlar (WebView2 baslar)
$appPid = $script:app.Id
function Family { $ids = @($appPid); $kids = Get-CimInstance Win32_Process | Where-Object { $_.ParentProcessId -eq $appPid -or $_.CommandLine -like "*kaiwen*dev.kaiwen.drop*" }; @($kids).Count }
$before = @(Get-Process kaiwen-drop -ErrorAction SilentlyContinue).Count
$wv = Family
Start-Process $script:Exe -ArgumentList "--quit" | Out-Null
$t = 0; while ($t -lt 10 -and (Get-Process -Id $appPid -ErrorAction SilentlyContinue)) { Start-Sleep -Milliseconds 500; $t += 0.5 }
$t2 = 0; while ($t2 -lt 10 -and (Family) -gt 0) { Start-Sleep -Milliseconds 500; $t2 += 0.5 }
$after = @(Get-Process kaiwen-drop -ErrorAction SilentlyContinue).Count
$wvAfter = Family
$ok = ($after -eq 0 -and $wvAfter -eq 0)
Write-Host "QUIT : $(if ($ok) { 'PASS' } else { 'FAIL' }) - once: app=$before webview-cocuk=$wv; sonra: app=$after webview-cocuk=$wvAfter (uygulama $t sn, WebView2 $t2 sn sonra kapandi)"
