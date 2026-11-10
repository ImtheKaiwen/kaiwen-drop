# E2E ortami: gecici dosya agaci + test ayarlari + uygulama baslatma.
# Gercek kullanici klasorlerine DOKUNMAZ. Mevcut ayar dosyasi yedeklenir; temizlikte geri konur.
. "$PSScriptRoot\lib.ps1"

$script:Base = Join-Path $env:TEMP "kd-e2e"
$script:Exe = (Resolve-Path (Join-Path $PSScriptRoot "..\..\src-tauri\target\debug\kaiwen-drop.exe")).Path
$script:Cfg = Join-Path $env:APPDATA "dev.kaiwen.drop\settings.json"
$script:CfgBak = "$script:Cfg.e2e-backup"

function New-Fixture {
  if (Test-Path $script:Base) { Remove-Item -Recurse -Force $script:Base }   # yalniz BU test agaci
  $dirs = @(
    "roots\Dev\active\projA\sub1", "roots\Dev\active\projA\sub2", "roots\Dev\notes",
    "roots\Docs\Personal", "roots\Docs\Partner", "roots\Docs\Together\Photos", "roots\Docs\Together\Memories",
    "roots\Docs\University", "roots\Docs\Archive", "files", "other"
  )
  foreach ($d in $dirs) { New-Item -ItemType Directory -Force (Join-Path $script:Base $d) | Out-Null }
  foreach ($f in @("a.txt", "b.txt", "c-türkçe-ğüşiöç.txt", "d.txt", "e.txt")) { Set-Content -Encoding UTF8 (Join-Path $script:Base "files\$f") "icerik $f" }
  Set-Content -Encoding UTF8 (Join-Path $script:Base "other\x.txt") "x"
}

function Write-TestSettings([hashtable]$Over = @{}) {
  New-Item -ItemType Directory -Force (Split-Path $script:Cfg) | Out-Null
  if ((Test-Path $script:Cfg) -and -not (Test-Path $script:CfgBak)) { Copy-Item $script:Cfg $script:CfgBak }
  $s = [ordered]@{
    version = 1
    roots = @(
      @{ id = "t-dev"; name = "Dev"; path = (Join-Path $script:Base "roots\Dev"); known = $null },
      @{ id = "t-docs"; name = "Documents"; path = (Join-Path $script:Base "roots\Docs"); known = $null }
    )
    language = "tr-TR"; theme = "light"; shortcut = "Ctrl+Alt+M"; density = "normal"
    show_hidden = $false; hide_dev_folders = $true; confirm_move = $true; corner_dock = $false
  }
  foreach ($k in $Over.Keys) { $s[$k] = $Over[$k] }
  [IO.File]::WriteAllText($script:Cfg, ($s | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding $false))
}

function Start-App {
  Get-Process kaiwen-drop -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 500
  $p = Start-Process $script:Exe -PassThru -RedirectStandardError (Join-Path $env:TEMP "kd-app.err") -RedirectStandardOutput (Join-Path $env:TEMP "kd-app.out")
  Start-Sleep -Milliseconds 2500
  $p
}

function Stop-App {
  Get-Process kaiwen-drop -ErrorAction SilentlyContinue | Stop-Process -Force
}

function Restore-Settings {
  if (Test-Path $script:CfgBak) { Move-Item -Force $script:CfgBak $script:Cfg } elseif (Test-Path $script:Cfg) { Remove-Item $script:Cfg }
}

function Close-Explorers([string]$Match) {
  $shell = New-Object -ComObject Shell.Application
  foreach ($w in @($shell.Windows())) { if ($w.LocationURL -like "*$Match*") { try { $w.Quit() } catch {} } }
}
