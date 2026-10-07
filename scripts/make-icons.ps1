# Regenerates the Windows .ico and Linux .png icons from assets/icon.png.
# Usage: powershell -ExecutionPolicy Bypass -File scripts/make-icons.ps1
# Requires Windows PowerShell 5.1 (System.Drawing). No ImageMagick needed.
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$root = Split-Path -Parent $PSScriptRoot
$srcPath = Join-Path $root "assets\icon.png"
$icoPath = Join-Path $root "assets\vidly.ico"
$png256  = Join-Path $root "assets\vidly-256.png"

if (-not (Test-Path -LiteralPath $srcPath)) {
    throw "missing source icon: $srcPath"
}

$sizes = 16, 32, 48, 64, 128, 256
$src = [System.Drawing.Image]::FromFile($srcPath)
$frames = @()
foreach ($s in $sizes) {
    $bmp = New-Object System.Drawing.Bitmap($s, $s)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.DrawImage($src, 0, 0, $s, $s)
    $g.Dispose()
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $frames += , @{ Size = $s; Data = $ms.ToArray() }
    $ms.Dispose()
    if ($s -eq 256) {
        $bmp.Save($png256, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    $bmp.Dispose()
}
$src.Dispose()

# ICONDIR header: reserved(2), type(2)=1, count(2)
$out = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter($out)
$bw.Write([UInt16]0)
$bw.Write([UInt16]1)
$bw.Write([UInt16]$frames.Count)
$offset = 6 + 16 * $frames.Count
foreach ($f in $frames) {
    $dim = if ($f.Size -ge 256) { 0 } else { $f.Size }
    $bw.Write([Byte]$dim)          # width
    $bw.Write([Byte]$dim)          # height
    $bw.Write([Byte]0)             # color count
    $bw.Write([Byte]0)             # reserved
    $bw.Write([UInt16]1)           # color planes
    $bw.Write([UInt16]32)          # bits per pixel
    $bw.Write([UInt32]$f.Data.Length)
    $bw.Write([UInt32]$offset)
    $offset += $f.Data.Length
}
foreach ($f in $frames) { $bw.Write($f.Data) }
$bw.Flush()
[System.IO.File]::WriteAllBytes($icoPath, $out.ToArray())
$bw.Dispose()
$out.Dispose()

Write-Host "generated: $icoPath"
Write-Host "generated: $png256"
