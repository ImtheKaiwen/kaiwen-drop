# W26 - birakma alani: gercek OLE surukle-birak (CF_HDROP) ile birakma -> hedef menusu; hedef secilmeden hicbir sey tasinmaz.
. "$PSScriptRoot\setup.ps1"
$ErrorActionPreference = "Stop"
$results = [ordered]@{}
function Record($id, $ok, $note) { $results[$id] = "$(if ($ok -eq $null) { 'NOT_VERIFIED' } elseif ($ok) { 'PASS' } else { 'FAIL' }) - $note"; Write-Host "$id : $($results[$id])" }
function Vis { @(Get-AppWindows $script:app.Id $true) }

Close-Explorers "kd-e2e"; Stop-App; New-Fixture; Write-TestSettings @{}
$script:app = Start-App
try {
  Start-Process $script:Exe -ArgumentList "--drop" | Out-Null; Start-Sleep -Milliseconds 2500
  $dropW = @(Vis)[0]
  Record "DROP" (@(Vis).Count -eq 1) "birakma alani acildi: $(@(Vis).Count) pencere, $($dropW.R - $dropW.L)x$($dropW.B - $dropW.T) px"
  [void](Save-Crop "19-drop-surface" ($dropW.L - 30) ($dropW.T - 30) (($dropW.R - $dropW.L) + 60) (($dropW.B - $dropW.T) + 60))

  $dragFile = Join-Path $script:Base "files\a.txt"
  $srcScript = Join-Path $env:TEMP "kd-dragsrc.ps1"
  @'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
$f = New-Object Windows.Forms.Form; $f.Text = 'kd-drag-src'; $f.StartPosition = 'Manual'; $f.Location = New-Object System.Drawing.Point(120, 140); $f.Size = New-Object System.Drawing.Size(220, 120); $f.TopMost = $true
$l = New-Object Windows.Forms.Label; $l.Text = 'surukle'; $l.Dock = 'Fill'; $l.TextAlign = 'MiddleCenter'
$l.Add_MouseDown({ $c = New-Object Collections.Specialized.StringCollection; [void]$c.Add($env:KD_FILE); $d = New-Object Windows.Forms.DataObject; $d.SetFileDropList($c); Add-Content "$env:TEMP\kd-dragsrc.log" "down"; $r = $l.DoDragDrop($d, [Windows.Forms.DragDropEffects]::Copy); Add-Content "$env:TEMP\kd-dragsrc.log" "effect=$r" })
$f.Controls.Add($l); [Windows.Forms.Application]::Run($f)
'@ | Set-Content -Encoding UTF8 $srcScript
  Remove-Item "$env:TEMP\kd-dragsrc.log" -ErrorAction SilentlyContinue
  $env:KD_FILE = $dragFile
  $src = Start-Process powershell -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-STA', '-File', $srcScript -PassThru
  Start-Sleep -Milliseconds 3000
  $sw = [W]::FindWindow([NullString]::Value, 'kd-drag-src')
  $r = New-Object W+RECT; [void][W]::GetWindowRect($sw, [ref]$r)
  $sx = [int](($r.L + $r.R) / 2); $sy = [int](($r.T + $r.B) / 2)
  $dx = [int](($dropW.L + $dropW.R) / 2); $dy = [int](($dropW.T + $dropW.B) / 2)
  Move-Mouse $sx $sy; Start-Sleep -Milliseconds 300
  [W]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
  for ($i = 1; $i -le 25; $i++) { Move-Mouse ([int]($sx + ($dx - $sx) * $i / 25)) ([int]($sy + ($dy - $sy) * $i / 25)); Start-Sleep -Milliseconds 40 }
  Start-Sleep -Milliseconds 500
  [void](Save-Crop "17-drag-hover" ($dropW.L - 30) ($dropW.T - 30) (($dropW.R - $dropW.L) + 60) (($dropW.B - $dropW.T) + 60))
  [W]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 1500
  $menus = @(Vis | Where-Object { $_.Hwnd -ne $dropW.Hwnd })
  Write-Host "kaynak log: $((Get-Content "$env:TEMP\kd-dragsrc.log" -ErrorAction SilentlyContinue) -join ',')"
  Record "W26a" ($menus.Count -eq 1) "birakma sonrasi hedef secme menusu: $($menus.Count)"
  [void](Save-Shot "18-drop-opens-menu")
  Record "W26b" (Test-Path $dragFile) "hedef secilmeden dosya tasinmadi (a.txt yerinde)"
  Press 0x1B; Start-Sleep -Milliseconds 800
  Record "W26c" ((Test-Path $dragFile) -and @(Vis | Where-Object { $_.Hwnd -ne $dropW.Hwnd }).Count -eq 0) "Esc: menu kapandi, dosya yerinde"
  # Esc ile birakma alanini kapat (alan odakta degilse X dugmesi)
  Stop-Process -Id $src.Id -Force -ErrorAction SilentlyContinue
} finally { Stop-App; Close-Explorers "kd-e2e"; Restore-Settings }
Write-Host "`n=== OZET ==="
$results.GetEnumerator() | ForEach-Object { "$($_.Key) : $($_.Value)" }
