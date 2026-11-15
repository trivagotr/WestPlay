[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$OutDirectory,[switch]$SelfTest)
$ErrorActionPreference='Stop'
$compiler=Join-Path $env:WINDIR 'Microsoft.NET\Framework64\v4.0.30319\csc.exe'
if (-not (Test-Path -LiteralPath $compiler)) { throw 'Windows x64 .NET Framework compiler is required.' }
$output=[IO.Path]::GetFullPath($OutDirectory)
New-Item -ItemType Directory -Path $output -Force | Out-Null
$program=Join-Path $output 'ConvoyReadOnly.exe'
& $compiler /nologo /target:exe /platform:x64 /optimize+ /r:System.Web.Extensions.dll "/out:$program" (Join-Path $PSScriptRoot 'LiveResearchReader.cs')
if ($LASTEXITCODE -ne 0) { throw 'Observer compilation failed.' }
if ($SelfTest) {
    & $program --self-test
    if ($LASTEXITCODE -ne 0) { throw 'Helper-owned-memory self-test failed.' }
}
Write-Output $program
