# Kaiwen Drop - tam Windows E2E (gercek Explorer, gercek tus/fare, gecici dosyalar).
# Kullanim: powershell -File tests\e2e\full.ps1 [-Only W12,W13]
param([string[]]$Only = @())
$Only = @($Only | ForEach-Object { $_ -split "," })
. "$PSScriptRoot\setup.ps1"
$ErrorActionPreference = "Stop"
$results = [ordered]@{}
function Record($id, $ok, $note) { $results[$id] = "$(if ($ok -eq $null) { 'NOT_VERIFIED' } elseif ($ok) { 'PASS' } else { 'FAIL' }) - $note"; Write-Host "$id : $($results[$id])" }
function Want($id) { $Only.Count -eq 0 -or $Only -contains $id }

$script:app = $null
function Vis { @(Get-AppWindows $script:app.Id $true) }
function VisCount { @(Get-AppWindows $script:app.Id $true).Count }
function Scale($w) { $w.Dpi / 96.0 }
function RowY($w, [int]$i, [bool]$afterAction = $false) {
  $s = Scale $w; [int]($w.T + (10 + 1 + 6 + $i * 34 + $(if ($afterAction -and $i -ge 1) { 9 } else { 0 }) + 17) * $s)
}
function CenterX($w) { [int](($w.L + $w.R) / 2) }
function Fresh([string]$Theme = "light", [hashtable]$Over = @{}) {
  Close-Explorers "kd-e2e"; Stop-App; New-Fixture
  $o = @{ theme = $Theme }; foreach ($k in $Over.Keys) { $o[$k] = $Over[$k] }
  Write-TestSettings $o
  $script:app = Start-App
}
function Open-Files($Names, [string]$Sub = "files") {
  $ex = Open-Explorer (Join-Path $script:Base $Sub); Start-Sleep -Milliseconds 700
  Focus-Hwnd $ex.HWND; Select-Items $ex $Names; Start-Sleep -Milliseconds 300; Focus-Hwnd $ex.HWND
  Move-Mouse 700 420
  $ex
}
function Open-Menu($ex) { Focus-Hwnd $ex.HWND; Move-Mouse 700 420; Send-Hotkey; Start-Sleep -Milliseconds 1000; Vis | Select-Object -First 1 }
# kok satirindan baslayarak yolu gez; son pencereyi dondur
function Walk($start, [string[]]$Path, $Index) {
  $cur = $start; $after = $false
  for ($k = 0; $k -lt $Path.Count; $k++) {
    $known = @((Vis) | ForEach-Object { $_.Hwnd })
    Move-Mouse (CenterX $cur) (RowY $cur $Index[$k] $after); Start-Sleep -Milliseconds 800
    $new = Vis | Where-Object { $known -notcontains $_.Hwnd } | Select-Object -First 1
    if (-not $new) { return $null }
    $cur = $new; $after = $true
  }
  $cur
}function Exists($rel) { Test-Path -LiteralPath (Join-Path $script:Base $rel) }

try {
# ------------------------------------------------------------------ W01 W02 W23
if (Want "W01") {
  Fresh
  Record "W23" ((VisCount) -eq 0) "tray-only baslangic, gorunur pencere: $((VisCount))"
  $ex = Open-Files @() ; Clear-Selection $ex; Focus-Hwnd $ex.HWND
  Send-Hotkey; Start-Sleep -Milliseconds 900
  Record "W01" ((VisCount) -eq 0) "Explorer secim yok, gorunur pencere: $((VisCount))"
  $np = Start-Process notepad -PassThru; Start-Sleep -Milliseconds 1200
  Send-Hotkey; Start-Sleep -Milliseconds 900
  Record "W02" ((VisCount) -eq 0) "Notepad on planda, gorunur pencere: $((VisCount))"
  Stop-Process $np -Force
}
# ------------------------------------------------------------------ W03 W04 W06 W07 W08 W09 W10 W11
if (Want "W03") {
  Fresh
  $ex = Open-Files @("a.txt", "b.txt", "c-türkçe-ğüşiöç.txt")
  $r = Open-Menu $ex
  Record "W03/W04" ((VisCount) -eq 1) "3 dosya secili, kok popup: $((VisCount))"
  [void](Save-Shot "01-root-popup-multiselect")
  # kok satirlari: Dev, Documents
  $c1 = Walk $r @("Documents") @(1)
  Record "W06" ($c1 -ne $null -and (VisCount) -eq 2) "Documents ustunde alt menu, pencere: $((VisCount))"
  Move-Mouse (CenterX $c1) (RowY $c1 2 $true); Start-Sleep -Milliseconds 500   # Partner'a gec
  Record "W07" ((VisCount) -ge 2) "fare alt menuye gecti, zincir pencere sayisi: $((VisCount))"
  Press 0x1B; Start-Sleep -Milliseconds 700
  Record "W08" ((VisCount) -eq 0) "Esc sonrasi gorunur: $((VisCount))"

  # W09 disari tiklama
  $r = Open-Menu $ex
  Click-At 1000 700; Start-Sleep -Milliseconds 700
  Record "W09" ((VisCount) -eq 0) "disari tiklama sonrasi gorunur: $((VisCount))"

  # W10 hizli tekrar
  Focus-Hwnd $ex.HWND; Send-Hotkey; Start-Sleep -Milliseconds 80; Send-Hotkey; Start-Sleep -Milliseconds 1000
  $n1 = (VisCount)
  if ($n1 -ge 1) { Press 0x1B; Start-Sleep -Milliseconds 700 }
  Record "W10" ($n1 -le 1 -and (VisCount) -eq 0) "hizli cift kisayol: $n1 pencere (>1 olmamali, ikinci basis ya yutulur ya oturumu kapatir); Esc sonrasi orphan: $((VisCount))"
  # W11 secim degisti: eski secim tasinmaz
  Select-Items $ex @("a.txt"); Start-Sleep -Milliseconds 300
  $r = Open-Menu $ex; Press 0x1B; Start-Sleep -Milliseconds 500
  Select-Items $ex @("b.txt"); Start-Sleep -Milliseconds 300; Focus-Hwnd $ex.HWND
  $r = Open-Menu $ex
  $c1 = Walk $r @("Documents") @(1)
  Move-Mouse (CenterX $c1) (RowY $c1 2 $true); Start-Sleep -Milliseconds 400
  # Partner altklasorsuz: tikla = tasi
  Click-At (CenterX $c1) (RowY $c1 2 $true); Start-Sleep -Milliseconds 900
  Press 0x0D; Start-Sleep -Milliseconds 1200
  Record "W11" ((Exists "roots\Docs\Partner\b.txt") -and (Exists "files\a.txt") -and -not (Exists "roots\Docs\Partner\a.txt")) "yalniz guncel secim (b.txt) tasindi, a.txt yerinde"
}
# ------------------------------------------------------------------ W12 W16 + kaskad ekran goruntuleri
if (Want "W12") {
  Fresh
  $ex = Open-Files @("a.txt", "c-türkçe-ğüşiöç.txt")
  $r = Open-Menu $ex
  $c1 = Walk $r @("Documents") @(1)
  $c2 = Walk $c1 @("Together") @(4)
  Record "W06c" ($c2 -ne $null -and (VisCount) -eq 3) "3 seviye kaskad: $((VisCount)) pencere"
  [void](Save-Shot "02-cascade-docs-together")
  # Photos (c2 icinde 2. satir, altklasorsuz) => tikla = tasi => onay diyalogu
  Click-At (CenterX $c2) (RowY $c2 2 $true); Start-Sleep -Milliseconds 900
  $dlg = @(Vis)
  Record "W12a" ($dlg.Count -eq 1) "onay diyalogu gorundu: $($dlg.Count) pencere"
  if ($dlg.Count -ge 1) { $d = $dlg[0]; [void](Save-Crop "03-confirm-dialog" ($d.L - 30) ($d.T - 30) (($d.R - $d.L) + 60) (($d.B - $d.T) + 60)) }
  Press 0x0D; Start-Sleep -Milliseconds 900
  [void](Save-Shot "04-toast-result")
  $t = @(Vis)
  Start-Sleep -Milliseconds 3500
  Record "W12" ((Exists "roots\Docs\Together\Photos\a.txt") -and -not (Exists "files\a.txt")) "dosya hedefte, kaynakta yok"
  Record "W16" (Exists "roots\Docs\Together\Photos\c-türkçe-ğüşiöç.txt") "Turkce karakterli dosya bozulmadan tasindi"
  Record "W30" ((VisCount) -eq 0) "islem sonrasi gorunur pencere: $((VisCount)); toast gorundu: $($t.Count -ge 1)"
}
# ------------------------------------------------------------------ W13 cakisma
if (Want "W13") {
  Fresh
  Set-Content (Join-Path $script:Base "roots\Docs\Together\Photos\a.txt") "ESKI-ICERIK"
  Set-Content (Join-Path $script:Base "other\a.txt") "YENI-ICERIK"
  $ex = Open-Files @("a.txt") "other"
  $r = Open-Menu $ex; $c1 = Walk $r @("Documents") @(1); $c2 = Walk $c1 @("Together") @(4)
  Click-At (CenterX $c2) (RowY $c2 2 $true); Start-Sleep -Milliseconds 900
  [void](Save-Shot "05-conflict-dialog")
  Press 0x1B; Start-Sleep -Milliseconds 900   # Iptal
  $old = (Get-Content (Join-Path $script:Base "roots\Docs\Together\Photos\a.txt") -Raw).Trim()
  Record "W13a" ($old -eq "ESKI-ICERIK" -and (Exists "other\a.txt")) "Iptal: hicbir sey degismedi, ustune yazilmadi"
  Focus-Hwnd $ex.HWND
  $r = Open-Menu $ex; $c1 = Walk $r @("Documents") @(1); $c2 = Walk $c1 @("Together") @(4)
  Click-At (CenterX $c2) (RowY $c2 2 $true); Start-Sleep -Milliseconds 900
  Press 0x0D; Start-Sleep -Milliseconds 1500    # birincil = Yeni adla tasi
  $old = (Get-Content (Join-Path $script:Base "roots\Docs\Together\Photos\a.txt") -Raw).Trim()
  $new = if (Exists "roots\Docs\Together\Photos\a (2).txt") { (Get-Content (Join-Path $script:Base "roots\Docs\Together\Photos\a (2).txt") -Raw).Trim() } else { "" }
  Record "W13" ($old -eq "ESKI-ICERIK" -and $new -eq "YENI-ICERIK" -and -not (Exists "other\a.txt")) "eski korundu, yeni 'a (2).txt' olarak tasindi"
}
# ------------------------------------------------------------------ W14 kendi icine tasima
if (Want "W14") {
  Fresh
  $ex = Open-Files @("projA") "roots\Dev\active"
  $r = Open-Menu $ex
  $c1 = Walk $r @("Dev") @(0)
  $c2 = Walk $c1 @("active") @(1)
  $c3 = Walk $c2 @("projA") @(1)
  $ok3 = $c3 -ne $null
  if ($ok3) {
    Click-At (CenterX $c3) (RowY $c3 1 $true); Start-Sleep -Milliseconds 900   # sub1: projA'nin ALTINA
    Start-Sleep -Milliseconds 600
    $vis = Vis
    if ($vis.Count -ge 1) { Press 0x0D; Start-Sleep -Milliseconds 800 }
  }
  Start-Sleep -Milliseconds 800
  Record "W14" ((Exists "roots\Dev\active\projA\sub1") -and -not (Exists "roots\Dev\active\projA\sub1\projA")) "klasor kendi altina tasinmadi (engellendi), derinlik: $ok3"
  [void](Save-Shot "06-self-move-blocked")
}
# ------------------------------------------------------------------ W15 kismi hata
if (Want "W15") {
  Fresh
  $locked = [IO.File]::Open((Join-Path $script:Base "files\d.txt"), 'Open', 'Read', 'None')
  try {
    $ex = Open-Files @("a.txt", "d.txt", "e.txt")
    $r = Open-Menu $ex; $c1 = Walk $r @("Documents") @(1)
    Click-At (CenterX $c1) (RowY $c1 3 $true); Start-Sleep -Milliseconds 900
    Press 0x0D; Start-Sleep -Milliseconds 1200
    [void](Save-Shot "07-partial-failure-toast")
  } finally { $locked.Dispose() }
  Start-Sleep -Milliseconds 5500
  Record "W15" ((Exists "roots\Docs\Personal\a.txt") -and (Exists "roots\Docs\Personal\e.txt") -and (Exists "files\d.txt") -and -not (Exists "roots\Docs\Personal\d.txt")) "3'ten 2'si tasindi, kilitli dosya yerinde ve kayip yok"
}
# ------------------------------------------------------------------ W27 sonradan eklenen klasor + klavye
if (Want "W27") {
  Fresh
  $ex = Open-Files @("a.txt")
  $r = Open-Menu $ex; $c1 = Walk $r @("Documents") @(1)
  $before = Vis; Press 0x1B; Start-Sleep -Milliseconds 600
  New-Item -ItemType Directory (Join-Path $script:Base "roots\Docs\Yeni Klasör") | Out-Null
  $r = Open-Menu $ex; $c1 = Walk $r @("Documents") @(1)
  $h = $c1.B - $c1.T
  Record "W27" ($c1 -ne $null -and $h -gt 0) "yeni klasor eklendi, menu yeniden acildi (yukseklik $h)"
  [void](Save-Shot "08-new-folder-visible")
  Press 0x1B; Start-Sleep -Milliseconds 600
  # klavye: Down Down (Documents) Right -> alt menu; Left -> kapanir; Esc
  Focus-Hwnd $ex.HWND; $r = Open-Menu $ex
  Press 0x28; Press 0x28; Start-Sleep -Milliseconds 200; Press 0x27; Start-Sleep -Milliseconds 900
  $k1 = (VisCount)
  Press 0x25; Start-Sleep -Milliseconds 600
  $k2 = (VisCount)
  Press 0x1B; Start-Sleep -Milliseconds 600
  Record "KBD" ($k1 -eq 2 -and $k2 -eq 1 -and (VisCount) -eq 0) "Right alt menu acti ($k1), Left kapatti ($k2), Esc hepsini ($((VisCount)))"
}
# ------------------------------------------------------------------ W29 koyu tema + Turkce
if (Want "W29") {
  Fresh "dark"
  $ex = Open-Files @("a.txt", "b.txt")
  $r = Open-Menu $ex; $c1 = Walk $r @("Documents") @(1); $c2 = Walk $c1 @("Together") @(4)
  [void](Save-Shot "09-dark-cascade")
  Record "W29a" ($c2 -ne $null) "koyu tema kaskad ekran goruntusu kaydedildi"
  Click-At (CenterX $c2) (RowY $c2 1 $true); Start-Sleep -Milliseconds 900
  $dlg = @(Vis)
  if ($dlg.Count -ge 1) { $d = $dlg[0]; [void](Save-Crop "10-dark-dialog" ($d.L - 30) ($d.T - 30) (($d.R - $d.L) + 60) (($d.B - $d.T) + 60)) }
  Press 0x1B; Start-Sleep -Milliseconds 700
}
# ------------------------------------------------------------------ W31 derin kaskad (Dev -> active -> projA -> alt klasorler)
if (Want "W31") {
  Fresh
  $ex = Open-Files @("a.txt", "b.txt", "c-türkçe-ğüşiöç.txt")
  $r = Open-Menu $ex
  $c1 = Walk $r @("Dev") @(0)
  $c2 = Walk $c1 @("active") @(1)
  $c3 = Walk $c2 @("projA") @(1)
  Record "W31" ($c3 -ne $null -and (VisCount) -eq 4) "Dev -> active -> projA derin kaskad: $((VisCount)) popup"
  [void](Save-Shot "02b-deep-cascade-dev-active-proj")
  Press 0x1B; Start-Sleep -Milliseconds 600
}
} finally {
  Stop-App; Close-Explorers "kd-e2e"; Restore-Settings
}
Write-Host "`n=== OZET ==="
$results.GetEnumerator() | ForEach-Object { "$($_.Key) : $($_.Value)" }
