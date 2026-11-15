# Windows PowerShell 5.1+. No administrator rights requested by this launcher.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$root = $PSScriptRoot
function Has-WebView2 {
    $id = '{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
    foreach ($key in @("HKCU:\Software\Microsoft\EdgeUpdate\Clients\$id", "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\$id", "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\$id")) {
        $pv = (Get-ItemProperty -LiteralPath $key -Name pv -ErrorAction SilentlyContinue).pv
        if ($pv -and $pv -ne '0.0.0.0') { return $true }
    }
    return $false
}
function Has-VcRuntime {
    # Built with MSVC 14.50.35717; Windows x64 files and x64 registration are required.
    $minimum = [Version]'14.50.35717.0'
    $base = [Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::LocalMachine, [Microsoft.Win32.RegistryView]::Registry64)
    try {
        $key = $base.OpenSubKey('SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64')
        if (-not $key) { return $false }
        try {
            if ($key.GetValue('Installed', 0) -ne 1) { return $false }
            $version = [Version]([string]$key.GetValue('Version', '0.0')).TrimStart('v')
            if ($version -lt $minimum) { return $false }
        } finally { $key.Dispose() }
    } finally { $base.Dispose() }
    $system64 = Join-Path $env:WINDIR 'System32'
    if (-not [Environment]::Is64BitProcess) { $system64 = Join-Path $env:WINDIR 'Sysnative' }
    foreach ($name in @('vcruntime140.dll', 'vcruntime140_1.dll')) {
        $path = Join-Path $system64 $name
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { return $false }
        $info = [Diagnostics.FileVersionInfo]::GetVersionInfo($path)
        $fileVersion = New-Object Version($info.FileMajorPart, $info.FileMinorPart, $info.FileBuildPart, $info.FilePrivatePart)
        if ($fileVersion -lt $minimum) { return $false }
    }
    return $true
}
function Download-File([string]$Url, [string]$Destination) {
    New-Item -ItemType Directory -Path (Split-Path -Parent $Destination) -Force | Out-Null
    Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $Destination
    if ((Get-Item -LiteralPath $Destination).Length -eq 0) { throw 'Indirme bos dosya dondurdu.' }
}
try {
    if (-not [Environment]::Is64BitOperatingSystem) { throw '64 bit Windows 10/11 gerekiyor.' }
    if (-not (Test-Path -LiteralPath (Join-Path $root 'westplay.exe') -PathType Leaf)) { throw 'Once ZIP dosyasinin tamamini bir klasore cikarin.' }
    $cache = Join-Path $env:LOCALAPPDATA 'WestPlay\runtime-downloads'
    if (-not (Has-WebView2)) {
        Write-Host 'Ilk kurulum: Microsoft WebView2 indiriliyor. Microsoft kurulum penceresini tamamlayin.'
        $setup = Join-Path $cache 'MicrosoftEdgeWebview2Setup.exe'
        Download-File 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' $setup
        $signature = Get-AuthenticodeSignature -LiteralPath $setup
        if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation(?:,|$)') { throw 'WebView2 Microsoft imzasi dogrulanamadi. Kurulum calistirilmadi.' }
        $process = Start-Process -FilePath $setup -ArgumentList '/install' -Wait -PassThru
        if (-not (Has-WebView2)) { throw "WebView2 kurulumu tamamlanmadi (kod $($process.ExitCode)). BASLAT.cmd dosyasini yeniden acin." }
    }
    if (-not (Has-VcRuntime)) {
        Write-Host 'Ilk kurulum: Microsoft Visual C++ x64 calisma zamani indiriliyor. Microsoft kurulum penceresini tamamlayin.'
        $vcSetup = Join-Path $cache 'vc_redist.x64.exe'
        Download-File 'https://aka.ms/vc14/vc_redist.x64.exe' $vcSetup
        $signature = Get-AuthenticodeSignature -LiteralPath $vcSetup
        if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation(?:,|$)') { throw 'Visual C++ Microsoft imzasi dogrulanamadi. Kurulum calistirilmadi.' }
        $process = Start-Process -FilePath $vcSetup -ArgumentList '/install', '/norestart' -Wait -PassThru
        if (-not (Has-VcRuntime)) { throw "Visual C++ x64 kurulumu tamamlanmadi (kod $($process.ExitCode)). Gerekirse Windows'u yeniden baslatip BASLAT.cmd dosyasini yeniden acin." }
    }
    $ffmpeg = Join-Path $root 'ffmpeg.exe'
    $expectedExe = '74DB6C184A03DBA2BDFE23E1A1F41CF5A8385BC1DE6A7A1B26DB1DC541ABEF93'
    $validFfmpeg = (Test-Path -LiteralPath $ffmpeg -PathType Leaf) -and ((Get-FileHash -LiteralPath $ffmpeg -Algorithm SHA256).Hash -eq $expectedExe)
    if (-not $validFfmpeg) {
        Write-Host 'Ilk kurulum: FFmpeg 8.0.1 resmi Gyan arsivinden indiriliyor (buyuk dosya; biraz surebilir).'
        $archive = Join-Path $cache 'ffmpeg-8.0.1-full_build.zip'
        $expectedArchive = '467CDE100A47ED4B03A897988AEB4A296890C1E2B2D2864204657D002BC5FB90'
        if (-not (Test-Path -LiteralPath $archive) -or (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ne $expectedArchive) {
            Download-File 'https://github.com/GyanD/codexffmpeg/releases/download/8.0.1/ffmpeg-8.0.1-full_build.zip' $archive
        }
        if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ne $expectedArchive) { throw 'FFmpeg arsivinin SHA256 kontrolu basarisiz. Dosya calistirilmadi.' }
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $zip = [IO.Compression.ZipFile]::OpenRead($archive)
        try {
            $licenseDir = Join-Path $root 'third-party\ffmpeg'
            New-Item -ItemType Directory -Path $licenseDir -Force | Out-Null
            foreach ($item in @(@('ffmpeg-8.0.1-full_build/bin/ffmpeg.exe', $ffmpeg), @('ffmpeg-8.0.1-full_build/LICENSE', (Join-Path $licenseDir 'LICENSE')), @('ffmpeg-8.0.1-full_build/README.txt', (Join-Path $licenseDir 'README.txt')))) {
                $entry = $zip.GetEntry($item[0])
                if (-not $entry) { throw "FFmpeg arsivinde gerekli dosya yok: $($item[0])" }
                [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $item[1], $true)
            }
        } finally { $zip.Dispose() }
        if ((Get-FileHash -LiteralPath $ffmpeg -Algorithm SHA256).Hash -ne $expectedExe) { throw 'FFmpeg programi SHA256 kontrolunu gecemedi.' }
    }
    Write-Host 'WestPlay aciliyor...'
    Start-Process -FilePath (Join-Path $root 'westplay.exe') -WorkingDirectory $root
} catch {
    Write-Host "Kurulum/acilis tamamlanamadi: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host 'Ilk kurulumda internet gerekir. Klasor yazilabilir olmali. Ayrintilar: START_HERE_TR.txt'
    Read-Host 'Pencereyi kapatmak icin Enter' | Out-Null
    exit 1
}
