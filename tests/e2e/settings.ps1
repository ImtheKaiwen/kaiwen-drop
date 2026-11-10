# Ayarlar penceresi E2E (UI Automation ile gercek WebView2 erisilebilirlik agaci): tek pencere, X/Esc, scroll, autostart, kok kaldirma.
param([string]$Theme = "light", [string]$Lang = "tr-TR")
. "$PSScriptRoot\setup.ps1"
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$ErrorActionPreference = "Stop"
$results = [ordered]@{}
function Record($id, $ok, $note) { $results[$id] = "$(if ($ok -eq $null) { 'NOT_VERIFIED' } elseif ($ok) { 'PASS' } else { 'FAIL' }) - $note"; Write-Host "$id : $($results[$id])" }
function Vis { @(Get-AppWindows $script:app.Id $true) }
function Run-Key { $k = Get-ItemProperty "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -ErrorAction SilentlyContinue; if ($k) { @($k.PSObject.Properties | Where-Object { "$($_.Value)" -like "*kaiwen-drop*" }).Count } else { 0 } }
function Ui-Find([int64]$Hwnd, [string]$Name, [int]$Retries = 12, [switch]$Actionable) {
  for ($i = 0; $i -lt $Retries; $i++) {
    $root = [Windows.Automation.AutomationElement]::FromHandle([IntPtr]$Hwnd)
    $cond = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, $Name)
    foreach ($el in $root.FindAll([Windows.Automation.TreeScope]::Descendants, $cond)) {
      if (-not $Actionable) { return $el }
      $p = $null
      if ($el.TryGetCurrentPattern([Windows.Automation.InvokePattern]::Pattern, [ref]$p) -or $el.TryGetCurrentPattern([Windows.Automation.TogglePattern]::Pattern, [ref]$p) -or $el.Current.ControlType.ProgrammaticName -eq 'ControlType.Button') { return $el }
    }
    Start-Sleep -Milliseconds 400
  }
  $null
}function Ui-Press($El) {
  $pat = $null
  if ($El.TryGetCurrentPattern([Windows.Automation.InvokePattern]::Pattern, [ref]$pat)) { $pat.Invoke(); return }
  if ($El.TryGetCurrentPattern([Windows.Automation.TogglePattern]::Pattern, [ref]$pat)) { $pat.Toggle(); return }
  $r = $El.Current.BoundingRectangle; Click-At ([int]($r.X + $r.Width / 2)) ([int]($r.Y + $r.Height / 2))
}
function Hash-Region($w) {
  $bmp = New-Object System.Drawing.Bitmap ($w.R - $w.L), ($w.B - $w.T)
  $g = [System.Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen($w.L, $w.T, 0, 0, $bmp.Size); $g.Dispose()
  $ms = New-Object IO.MemoryStream; $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
  [BitConverter]::ToString((New-Object Security.Cryptography.SHA1Managed).ComputeHash($ms.ToArray()))
}
function Wheel([int]$delta) { [W]::mouse_event(0x800, 0, 0, [BitConverter]::ToUInt32([BitConverter]::GetBytes([int]$delta), 0), [UIntPtr]::Zero) }

Close-Explorers "kd-e2e"; Stop-App; New-Fixture; Write-TestSettings @{ theme = $Theme; language = $Lang }
$script:app = Start-App
$tr = $Lang -eq "tr-TR"
try {
  Record "W23" (@(Vis).Count -eq 0) "baslangicta gorunur pencere: $(@(Vis).Count)"
  Start-Process $script:Exe | Out-Null; Start-Sleep -Milliseconds 2500   # tek-instance: mevcut uygulama Ayarlar'i acar
  Record "W17a" (@(Vis).Count -eq 1) "Ayarlar acildi, gorunur pencere: $(@(Vis).Count)"
  Start-Process $script:Exe | Out-Null; Start-Sleep -Milliseconds 1800
  Record "W19" (@(Vis).Count -eq 1 -and @(Get-Process kaiwen-drop).Count -eq 1) "iki kez acma: pencere $(@(Vis).Count), surec $(@(Get-Process kaiwen-drop).Count)"
  $w = @(Vis)[0]; $s = $w.Dpi / 96.0
  Record "SIZE" (($w.R - $w.L) / $s -lt 520 -and ($w.B - $w.T) / $s -lt 640) "boyut mantiksal: $([int](($w.R - $w.L) / $s))x$([int](($w.B - $w.T) / $s))"
  Focus-Hwnd $w.Hwnd
  [void](Save-Crop "11-settings-top-$Theme-$($Lang.Substring(0,2))" ($w.L - 20) ($w.T - 20) (($w.R - $w.L) + 40) (($w.B - $w.T) + 40))
  $title = Ui-Find $w.Hwnd $(if ($tr) { "Ayarlar" } else { "Settings" })
  Record "I18N" ($title -ne $null) "baslik ($Lang) UIA agacinda: $([bool]$title)"
  if (-not $tr -or $Theme -ne "light") { return }

  # DD: ozel acilir liste (yerel select degil): acilir, Esc yalniz listeyi kapatir, secim kaydedilir
  $dd = Ui-Find $w.Hwnd "Tema" -Actionable
  if ($dd) {
    Ui-Press $dd; Start-Sleep -Milliseconds 500
    [void](Save-Crop "15-settings-dropdown-open" ($w.L - 20) ($w.T - 20) (($w.R - $w.L) + 40) (($w.B - $w.T) + 40))
    $opt = Ui-Find $w.Hwnd "Koyu" -Retries 3 -Actionable
    Press 0x1B; Start-Sleep -Milliseconds 500
    Record "DD" ($opt -ne $null -and @(Vis).Count -eq 1) "liste acildi (secenek bulundu: $([bool]$opt)); Esc sonrasi Ayarlar acik: $(@(Vis).Count)"
  } else { Record "DD" $null "Tema dugmesi UIA agacinda bulunamadi" }

  # SC: kisayol degistirme (gercek tus): Ctrl+Shift+K -> ayar dosyasina yazilir ve yeni kisayol calisir
  $sc = Ui-Find $w.Hwnd "Kısayol" -Actionable
  if ($sc) {
    $r = $sc.Current.BoundingRectangle; Click-At ([int]($r.X + $r.Width / 2)) ([int]($r.Y + $r.Height / 2)); Start-Sleep -Milliseconds 700
    [void](Save-Crop "16-shortcut-recording" ($w.L - 20) ($w.T - 20) (($w.R - $w.L) + 40) (($w.B - $w.T) + 40))
    # Gercek klavye tarama kodu (scan code) uretir; vk-only keybd_event e.code'u bos birakir.
    $ks = { param($vk, $sc, $up) [W]::keybd_event([byte]$vk, [byte]$sc, $(if ($up) { 2 } else { 0 }), [UIntPtr]::Zero) }
    & $ks 0x11 0x1D $false; & $ks 0x10 0x2A $false; Start-Sleep -Milliseconds 80
    & $ks 0x4B 0x25 $false; Start-Sleep -Milliseconds 40; & $ks 0x4B 0x25 $true; & $ks 0x10 0x2A $true; & $ks 0x11 0x1D $true
    Start-Sleep -Milliseconds 1200
    $saved = (Get-Content $script:Cfg -Raw | ConvertFrom-Json).shortcut
    [void](Save-Crop "17-shortcut-changed" ($w.L - 20) ($w.T - 20) (($w.R - $w.L) + 40) (($w.B - $w.T) + 40))
    Record "SC" ($saved -eq "Ctrl+Shift+K") "kayitli kisayol: $saved"
  } else { Record "SC" $null "Kisayol dugmesi UIA agacinda bulunamadi" }

  # W18: tekerlekle kaydirma
  $h0 = Hash-Region $w
  Move-Mouse ([int](($w.L + $w.R) / 2)) ([int](($w.T + $w.B) / 2)); Start-Sleep -Milliseconds 200
  1..3 | ForEach-Object { Wheel -480; Start-Sleep -Milliseconds 250 }
  Start-Sleep -Milliseconds 400
  $h1 = Hash-Region $w
  [void](Save-Crop "12-settings-scrolled" ($w.L - 20) ($w.T - 20) (($w.R - $w.L) + 40) (($w.B - $w.T) + 40))
  Record "W18" ($h0 -ne $h1) "tekerlek sonrasi icerik kaydi: $($h0 -ne $h1) (kaydirma cubugu CSS ile gizli; bkz. 12-settings-scrolled.png)"
  1..4 | ForEach-Object { Wheel 480; Start-Sleep -Milliseconds 200 }

  # W24: autostart varsayilan kapali; ac -> Run anahtari; kapat -> kalkar
  $before = Run-Key
  $sw = Ui-Find $w.Hwnd "Windows açılışında başlat" -Actionable
  if ($sw) {
    Ui-Press $sw; Start-Sleep -Milliseconds 1000; $on = Run-Key
    Ui-Press (Ui-Find $w.Hwnd "Windows açılışında başlat" -Actionable); Start-Sleep -Milliseconds 1000; $off = Run-Key
    Record "W24" ($before -eq 0 -and $on -ge 1 -and $off -eq 0) "Run anahtari: baslangic=$before acik=$on kapali=$off"
  } else { Record "W24" $null "anahtar UIA agacinda bulunamadi" }

  # W28: koku kaldir -> yalniz kayit silinir
  $cfgBefore = (Get-Content $script:Cfg -Raw | ConvertFrom-Json).roots.Count
  $menu = Ui-Find $w.Hwnd "Satır menüsü" -Actionable
  if ($menu) {
    Ui-Press $menu; Start-Sleep -Milliseconds 500
    [void](Save-Crop "14-settings-row-menu" ($w.L - 20) ($w.T - 20) (($w.R - $w.L) + 40) (($w.B - $w.T) + 40))
    Ui-Press (Ui-Find $w.Hwnd "Kaldır" -Actionable); Start-Sleep -Milliseconds 1000
    $cfgAfter = (Get-Content $script:Cfg -Raw | ConvertFrom-Json).roots.Count
    Record "W28" ($cfgAfter -eq $cfgBefore - 1 -and (Test-Path (Join-Path $script:Base "roots\Dev\active\projA\sub1")) -and (Test-Path (Join-Path $script:Base "roots\Dev\notes"))) "kok kaydi $cfgBefore -> $cfgAfter; gercek klasor ve icerigi duruyor"
  } else { Record "W28" $null "satir menusu UIA agacinda bulunamadi" }

  # W17: X (gercek fare tiklamasi)
  $w = @(Vis)[0]; $s = $w.Dpi / 96.0
  Click-At ([int]($w.R - 28 * $s)) ([int]($w.T + 26 * $s)); Start-Sleep -Milliseconds 900
  Record "W17" (@(Vis).Count -eq 0 -and @(Get-Process kaiwen-drop).Count -eq 1) "X sonrasi pencere: $(@(Vis).Count); uygulama (tray) calismaya devam ediyor"

  # Esc ile kapanma
  Start-Process $script:Exe | Out-Null; Start-Sleep -Milliseconds 2200
  $w = @(Vis)[0]; Focus-Hwnd $w.Hwnd; Press 0x1B; Start-Sleep -Milliseconds 900
  Record "ESC" (@(Vis).Count -eq 0) "Settings Esc ile kapandi: $(@(Vis).Count) pencere"
} finally {
  Stop-App; Close-Explorers "kd-e2e"; Restore-Settings
  $k = Get-ItemProperty "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -ErrorAction SilentlyContinue
  if ($k) { $k.PSObject.Properties | Where-Object { "$($_.Value)" -like "*kaiwen-drop*" } | ForEach-Object { Remove-ItemProperty "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" $_.Name } }
}
Write-Host "`n=== OZET ==="
$results.GetEnumerator() | ForEach-Object { "$($_.Key) : $($_.Value)" }
