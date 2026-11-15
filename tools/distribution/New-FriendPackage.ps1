[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$RepoRoot,
    [Parameter(Mandatory=$true)][string]$BinaryDirectory,
    [Parameter(Mandatory=$true)][string]$OutputDirectory,
    [Parameter(Mandatory=$true)][string]$ApkPath,
    [string]$LicenseDirectory,
    [string]$AndroidNoticesPath,
    [switch]$CreateZip
)
$ErrorActionPreference = 'Stop'
function Require-File([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf) -or (Get-Item -LiteralPath $Path).Length -eq 0) { throw "Required file missing/empty: $Path" }
}
$RepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path
$BinaryDirectory = (Resolve-Path -LiteralPath $BinaryDirectory).Path
$ApkPath = (Resolve-Path -LiteralPath $ApkPath).Path
if (-not $AndroidNoticesPath) { $AndroidNoticesPath = Join-Path (Split-Path -Parent $ApkPath) 'WestPlay-Phone-Third-Party-Notices.md' }
Require-File $AndroidNoticesPath
if (-not $LicenseDirectory) { $LicenseDirectory = $RepoRoot }
$LicenseDirectory = (Resolve-Path -LiteralPath $LicenseDirectory).Path
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $OutputDirectory) { throw 'OutputDirectory must be a NEW directory; existing content is never overwritten.' }
if ($OutputDirectory.StartsWith($RepoRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'OutputDirectory must be outside the source repository.' }
$files = @('westplay.exe', 'westplay.dll', 'westplay-probe.exe', 'westplay-maps.exe')
foreach ($name in $files) { Require-File (Join-Path $BinaryDirectory $name) }
foreach ($name in @('LICENSE', 'THIRD_PARTY_LICENSES.md', 'THIRD_PARTY_NOTICES.md')) { Require-File (Join-Path $LicenseDirectory $name) }
Require-File $ApkPath
Require-File (Join-Path $RepoRoot 'ui\dist\index.html')
Require-File (Join-Path $RepoRoot 'ui\dist\cabin.html')
Require-File (Join-Path $RepoRoot 'docs\friend-package.tr.md')
foreach ($name in @('BASLAT.cmd', 'Start-WestPlay.ps1')) { Require-File (Join-Path $PSScriptRoot $name) }
$zipPath = $OutputDirectory + '.zip'
if ($CreateZip -and (Test-Path -LiteralPath $zipPath)) { throw "ZIP already exists: $zipPath" }
# Snapshot working source, including untracked source files, without .git, build caches or local keys.
$sourceFiles = @(& git -C $RepoRoot ls-files --cached --others --exclude-standard)
if ($LASTEXITCODE -ne 0 -or $sourceFiles.Count -eq 0) { throw 'Cannot enumerate source files with git.' }
$denySegment = '(^|/)(\.git|\.codex|\.agents|node_modules|target|build|dist|\.gradle|\.idea|outputs|\.android|\.signing|signing|cache|caches)(/|$)'
$denyName = '(^|/)(\.env[^/]*|local\.properties|signing\.properties|credentials(?:[._-][^/]*)?|secrets(?:[._-][^/]*)?|.*\.(jks|keystore|p12|pfx|pem|key|apk|aab|exe|dll|zip|7z|log))$'
$sourceFiles = @($sourceFiles | Where-Object { $_ -notmatch $denySegment -and $_ -notmatch $denyName -and (Test-Path -LiteralPath (Join-Path $RepoRoot $_) -PathType Leaf) } | Sort-Object -Unique)
New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
foreach ($name in $files) { Copy-Item -LiteralPath (Join-Path $BinaryDirectory $name) -Destination $OutputDirectory }
foreach ($name in @('LICENSE', 'THIRD_PARTY_LICENSES.md', 'THIRD_PARTY_NOTICES.md')) { Copy-Item -LiteralPath (Join-Path $LicenseDirectory $name) -Destination $OutputDirectory }
if (Test-Path -LiteralPath (Join-Path $LicenseDirectory 'third-party') -PathType Container) { Copy-Item -LiteralPath (Join-Path $LicenseDirectory 'third-party') -Destination $OutputDirectory -Recurse }
New-Item -ItemType Directory -Path (Join-Path $OutputDirectory 'ui'), (Join-Path $OutputDirectory 'Android'), (Join-Path $OutputDirectory 'source') | Out-Null
# Runtime resource lookup expects ui/index.html next to the application.
Copy-Item -Path (Join-Path $RepoRoot 'ui\dist\*') -Destination (Join-Path $OutputDirectory 'ui') -Recurse
Copy-Item -LiteralPath $ApkPath -Destination (Join-Path $OutputDirectory 'Android\WestPlay-Phone-Preview.apk')
Copy-Item -LiteralPath $AndroidNoticesPath -Destination (Join-Path $OutputDirectory 'Android\THIRD_PARTY_NOTICES.md')
foreach ($name in @('BASLAT.cmd', 'Start-WestPlay.ps1')) { Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination $OutputDirectory }
Copy-Item -LiteralPath (Join-Path $RepoRoot 'docs\friend-package.tr.md') -Destination (Join-Path $OutputDirectory 'START_HERE_TR.txt')
# Zip directly from the allowlisted working files: no temporary source tree or destructive cleanup.
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$sourceZip = [IO.Compression.ZipFile]::Open((Join-Path $OutputDirectory 'source\WestPlay-source.zip'), [IO.Compression.ZipArchiveMode]::Create)
try {
    foreach ($relative in $sourceFiles) { [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($sourceZip, (Join-Path $RepoRoot $relative), $relative.Replace('\','/'), [IO.Compression.CompressionLevel]::Optimal) | Out-Null }
} finally { $sourceZip.Dispose() }
@'
FFmpeg is NOT redistributed in this ZIP. First launch downloads the original Gyan 8.0.1 full archive directly to the user's PC.
Pinned archive SHA256: 467CDE100A47ED4B03A897988AEB4A296890C1E2B2D2864204657D002BC5FB90
Manifest: https://github.com/microsoft/winget-pkgs/blob/master/manifests/g/Gyan/FFmpeg/8.0.1/Gyan.FFmpeg.installer.yaml
Origin: https://github.com/GyanD/codexffmpeg/releases/download/8.0.1/ffmpeg-8.0.1-full_build.zip
FFmpeg GPLv3 license and Gyan README are extracted to third-party/ffmpeg on first launch.
Do not redistribute a used folder containing ffmpeg.exe without fulfilling corresponding-source obligations.
WebView2 is downloaded from Microsoft's official bootstrapper link if missing, its Authenticode signature is verified, and its installer is shown.
Visual C++ x64 runtime is downloaded directly from Microsoft https://aka.ms/vc14/vc_redist.x64.exe when required. Microsoft Authenticode is verified before its visible installer runs. Minimum runtime version: 14.50.35717.0 (MSVC toolset used for these binaries).
Visual C++ documentation: https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist
WebView2 distribution documentation: https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution
WestPlay working-source snapshot: source/WestPlay-source.zip. Cargo.lock/package-lock files identify dependencies; build instructions are in its README.
'@ | Set-Content -LiteralPath (Join-Path $OutputDirectory 'RUNTIME_SOURCES.txt') -Encoding UTF8
# Include no absolute build paths or device credentials in the share manifest.
$entries = @(Get-ChildItem -LiteralPath $OutputDirectory -File -Recurse | ForEach-Object { [ordered]@{ path = $_.FullName.Substring($OutputDirectory.Length + 1).Replace('\','/'); bytes = $_.Length; sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() } })
$entries | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'SHA256.json') -Encoding UTF8
if (Test-Path -LiteralPath (Join-Path $OutputDirectory 'ffmpeg.exe')) { throw 'The share package must not contain a downloaded FFmpeg binary.' }
if ($CreateZip) {
    [IO.Compression.ZipFile]::CreateFromDirectory($OutputDirectory, $zipPath, [IO.Compression.CompressionLevel]::Optimal, $true)
    Write-Output $zipPath
}
Write-Output $OutputDirectory
