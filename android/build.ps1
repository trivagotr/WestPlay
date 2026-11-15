param([string]$Sdk = 'D:/Android/Sdk')
$ErrorActionPreference = 'Stop'
$work = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$env:ANDROID_HOME = $Sdk
$env:ANDROID_USER_HOME = Join-Path $work 'android-work/user'
$env:GRADLE_USER_HOME = Join-Path $work 'android-work/gradle'
$env:WESTPLAY_DEBUG_KEY = Join-Path $env:ANDROID_USER_HOME 'debug.keystore'
New-Item -ItemType Directory -Force -Path $env:ANDROID_USER_HOME | Out-Null
$gradle = Join-Path $work 'toolchains/gradle-8.13/bin/gradle.bat'
if (!(Test-Path -LiteralPath $gradle)) {
  $archive = Join-Path $work 'gradle-8.13-bin.zip'
  Invoke-WebRequest 'https://services.gradle.org/distributions/gradle-8.13-bin.zip' -OutFile $archive
  $expected = (Invoke-RestMethod 'https://services.gradle.org/distributions/gradle-8.13-bin.zip.sha256').Trim()
  if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) { throw 'Gradle SHA256 mismatch' }
  Expand-Archive -LiteralPath $archive -DestinationPath (Join-Path $work 'toolchains') -Force
}
Push-Location $PSScriptRoot
try { & $gradle --no-daemon testDebugUnitTest assembleDebug lintDebug; if ($LASTEXITCODE -ne 0) { throw 'Android verification failed' } } finally { Pop-Location }
