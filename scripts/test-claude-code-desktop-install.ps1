param(
  [string]$InstallerPath = (Join-Path $env:TEMP "agenticboot\claude-desktop-setup.exe")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Phase {
  param([string]$Message)
  Write-Host "[phase] $Message"
}

function Write-CommandLine {
  param([string]$CommandLine)
  Write-Host "[command] $CommandLine"
}

function Write-ResultLine {
  param([string]$Message)
  Write-Host "[result] $Message"
}

$wingetArgs = @(
  "install",
  "--id",
  "Anthropic.Claude",
  "-e",
  "--accept-package-agreements",
  "--accept-source-agreements"
)

$wingetCommandLine = "winget " + ($wingetArgs -join " ")
$winget = Get-Command winget -ErrorAction SilentlyContinue

if ($winget) {
  Write-Phase "trying winget install"
  Write-CommandLine $wingetCommandLine
  & $winget.Source @wingetArgs
  $wingetExit = $LASTEXITCODE

  if ($wingetExit -eq 0) {
    Write-ResultLine "winget install completed"
    exit 0
  }

  Write-ResultLine "winget install failed with exit code $wingetExit; falling back to the official installer"
} else {
  Write-ResultLine "winget is unavailable; falling back to the official installer"
}

$architecture = if (($env:PROCESSOR_ARCHITECTURE -eq "ARM64") -or ($env:PROCESSOR_ARCHITEW6432 -eq "ARM64")) {
  "arm64"
} else {
  "x64"
}

$downloadUrl = "https://claude.ai/api/desktop/win32/$architecture/setup/latest/redirect"
$installerDir = Split-Path -Path $InstallerPath -Parent
if (-not (Test-Path -LiteralPath $installerDir)) {
  New-Item -ItemType Directory -Path $installerDir -Force | Out-Null
}

Write-Phase "downloading official installer"
Write-CommandLine "Invoke-WebRequest -Uri `"$downloadUrl`" -OutFile `"$InstallerPath`""
Invoke-WebRequest -Uri $downloadUrl -OutFile $InstallerPath

Write-Phase "launching downloaded installer"
Write-CommandLine $InstallerPath
& $InstallerPath
$installerExit = $LASTEXITCODE

if ($installerExit -ne 0) {
  throw "Claude desktop installer exited with code $installerExit"
}

Write-ResultLine "fallback installer completed"
