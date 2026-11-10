# Kaiwen Drop - gercek Windows E2E yardimcilari.
# YALNIZ test amacli sahte girdi (SendInput benzeri) kullanir; uygulamanin kendisi asla kullanmaz.
# Tum dosya islemleri %TEMP%\kd-e2e altindaki gecici klasorlerde yapilir.

Add-Type -TypeDefinition @"
using System; using System.Runtime.InteropServices; using System.Text; using System.Collections.Generic;
public class W {
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern bool GetCursorPos(out POINT p);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern IntPtr FindWindow(string cls, string title);
  [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool attach);
  [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();

  public static string[] Windows(uint pid, bool onlyVisible) {
    var list = new List<string>();
    EnumWindows((h, l) => {
      uint p; GetWindowThreadProcessId(h, out p);
      if (p != pid) return true;
      bool vis = IsWindowVisible(h);
      if (onlyVisible && !vis) return true;
      RECT r; GetWindowRect(h, out r);
      var c = new StringBuilder(256); GetClassName(h, c, 256);
      var t = new StringBuilder(256); GetWindowText(h, t, 256);
      list.Add(h.ToInt64() + "|" + c + "|" + (vis ? 1 : 0) + "|" + r.L + "," + r.T + "," + r.R + "," + r.B + "|" + GetDpiForWindow(h) + "|" + t);
      return true;
    }, IntPtr.Zero);
    return list.ToArray();
  }

  public static void Focus(IntPtr h) {
    IntPtr fg = GetForegroundWindow(); uint pid;
    uint fgT = GetWindowThreadProcessId(fg, out pid); uint me = GetCurrentThreadId();
    AttachThreadInput(me, fgT, true); ShowWindow(h, 9); SetForegroundWindow(h); AttachThreadInput(me, fgT, false);
  }
}
"@
[void][W]::SetProcessDpiAwarenessContext([IntPtr]-4)
Add-Type -AssemblyName System.Drawing, System.Windows.Forms

$script:Out = Join-Path $PSScriptRoot "..\..\docs\screenshots"
New-Item -ItemType Directory -Force $script:Out | Out-Null

function Get-AppWindows([int]$AppPid, [bool]$Visible = $true) {
  [W]::Windows([uint32]$AppPid, $Visible) | ForEach-Object {
    $p = $_ -split '\|', 6
    $r = $p[3] -split ','
    [pscustomobject]@{ Hwnd = [int64]$p[0]; Class = $p[1]; Visible = ($p[2] -eq '1'); L = [int]$r[0]; T = [int]$r[1]; R = [int]$r[2]; B = [int]$r[3]; Dpi = [int]$p[4]; Title = $p[5] }
  } | Where-Object { $_.Class -eq 'Tauri Window' }

}

function Key([byte]$vk, [bool]$down) { [W]::keybd_event($vk, 0, $(if ($down) { 0 } else { 2 }), [UIntPtr]::Zero) }
function Send-Hotkey {
  Key 0x11 $true; Key 0x12 $true; Key 0x4D $true; Start-Sleep -Milliseconds 60
  Key 0x4D $false; Key 0x12 $false; Key 0x11 $false
}
function Press([byte]$vk) { Key $vk $true; Start-Sleep -Milliseconds 40; Key $vk $false }
function Move-Mouse([int]$x, [int]$y) { [void][W]::SetCursorPos($x, $y) }
function Click-At([int]$x, [int]$y) {
  Move-Mouse $x $y; Start-Sleep -Milliseconds 80
  [W]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 50; [W]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero)
}
function Get-Cursor { $p = New-Object W+POINT; [void][W]::GetCursorPos([ref]$p); @($p.X, $p.Y) }

function Save-Shot([string]$Name) {
  # Gizlilik: tam ekran yerine uygulama pencerelerinin birlesimi (+ kenar) kirpilir; baska uygulamalar kadraja girmez.
  if ($script:app) {
    $ws = @(Get-AppWindows $script:app.Id $true)
    if ($ws.Count -gt 0) {
      $m = 60
      $l = [Math]::Max(0, ($ws | Measure-Object L -Minimum).Minimum - $m); $t = [Math]::Max(0, ($ws | Measure-Object T -Minimum).Minimum - $m)
      $r = ($ws | Measure-Object R -Maximum).Maximum + $m; $bt = ($ws | Measure-Object B -Maximum).Maximum + $m
      return (Save-Crop $Name $l $t ($r - $l) ($bt - $t))
    }
    # Uygulama penceresi yok (ornegin bos secim): yalniz on plandaki test Explorer penceresini kirp; baska bir sey ise ekran goruntusu alma.
    $fg = [W]::GetForegroundWindow(); $cn = New-Object System.Text.StringBuilder 64; [void][W]::GetClassName($fg, $cn, 64)
    if ("$cn" -eq "CabinetWClass") {
      $rr = New-Object W+RECT; [void][W]::GetWindowRect($fg, [ref]$rr)
      return (Save-Crop $Name ([Math]::Max(0, $rr.L)) ([Math]::Max(0, $rr.T)) ($rr.R - $rr.L) ($rr.B - $rr.T))
    }
    return $null
  }
  $b = [System.Windows.Forms.SystemInformation]::VirtualScreen
  $bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($b.Left, $b.Top, 0, 0, $bmp.Size)
  $path = Join-Path $script:Out "$Name.png"
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $bmp.Dispose()
  $path
}
function Save-Crop([string]$Name, [int]$L, [int]$T, [int]$W_, [int]$H_) {
  $bmp = New-Object System.Drawing.Bitmap $W_, $H_
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($L, $T, 0, 0, $bmp.Size)
  $path = Join-Path $script:Out "$Name.png"
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $bmp.Dispose()
  $path
}

# Explorer penceresi ac, foreground yap, verilen ogeleri sec (yalniz test; hicbir dosya degistirilmez).
function Open-Explorer([string]$Folder) {
  $shell = New-Object -ComObject Shell.Application
  $before = @($shell.Windows() | ForEach-Object { $_.HWND })
  Start-Process explorer.exe -ArgumentList "`"$Folder`""
  for ($i = 0; $i -lt 40; $i++) {
    Start-Sleep -Milliseconds 250
    $w = $shell.Windows() | Where-Object { $before -notcontains $_.HWND -and $_.LocationURL } | Select-Object -First 1
    if ($w) { return $w }
  }
  throw "Explorer penceresi acilamadi"
}
function Select-Items($ExplorerWindow, [string[]]$Names) {
  $folder = $ExplorerWindow.Document.Folder
  $first = $true
  foreach ($n in $Names) {
    $item = $folder.ParseName($n)
    $flags = if ($first) { 1 -bor 4 -bor 8 -bor 16 } else { 1 -bor 8 }
    $ExplorerWindow.Document.SelectItem($item, $flags); $first = $false
  }
}
function Clear-Selection($ExplorerWindow) {
  # Secili ogeleri tek tek SVSI_DESELECT (0) ile kaldir.
  foreach ($it in @($ExplorerWindow.Document.SelectedItems())) { $ExplorerWindow.Document.SelectItem($it, 0) }
}
function Focus-Hwnd([int64]$Hwnd) { [W]::Focus([IntPtr]$Hwnd); Start-Sleep -Milliseconds 400 }

