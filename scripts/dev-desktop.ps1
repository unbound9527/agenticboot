# Requires -Version 5.1

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-ProjectRoot {
  return (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
}

function Get-NodeVersion {
  param(
    [Parameter(Mandatory = $true)]
    [string]$ProjectRoot
  )

  $versionFile = Join-Path $ProjectRoot ".node-version"
  if (-not (Test-Path $versionFile)) {
    throw "Missing .node-version at $versionFile"
  }

  $version = (Get-Content -Raw $versionFile).Trim()
  if (-not $version) {
    throw ".node-version is empty"
  }

  return $version
}

function Get-ManagedNodeRoot {
  if ($env:AGENTICBOOT_MANAGED_NODE_ROOT) {
    return $env:AGENTICBOOT_MANAGED_NODE_ROOT
  }

  return Join-Path $env:LOCALAPPDATA "AgenticBoot\managed-node"
}

function Ensure-ManagedNode {
  param(
    [Parameter(Mandatory = $true)]
    [string]$NodeVersion,
    [Parameter(Mandatory = $true)]
    [string]$ManagedRoot
  )

  $nodeDirName = "node-v$NodeVersion-win-x64"
  $nodeDir = Join-Path $ManagedRoot $nodeDirName
  $nodeExe = Join-Path $nodeDir "node.exe"

  if (Test-Path $nodeExe) {
    return $nodeDir
  }

  New-Item -ItemType Directory -Path $ManagedRoot -Force | Out-Null

  $zipPath = Join-Path $ManagedRoot "$nodeDirName.zip"
  $downloadUrl = "https://nodejs.org/dist/v$NodeVersion/$nodeDirName.zip"

  Write-Host "Downloading Node.js $NodeVersion from $downloadUrl"
  Invoke-WebRequest $downloadUrl -OutFile $zipPath

  Write-Host "Extracting Node.js $NodeVersion to $ManagedRoot"
  Expand-Archive -Path $zipPath -DestinationPath $ManagedRoot -Force

  if (-not (Test-Path $nodeExe)) {
    throw "Managed Node.js installation failed: $nodeExe not found"
  }

  return $nodeDir
}

function Invoke-ManagedCommand {
  param(
    [Parameter(Mandatory = $true)]
    [string]$FilePath,
    [Parameter()]
    [string[]]$Arguments = @(),
    [Parameter(Mandatory = $true)]
    [string]$WorkingDirectory
  )

  & $FilePath @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "Command failed with exit code ${LASTEXITCODE}: $FilePath $($Arguments -join ' ')"
  }
}

$projectRoot = Get-ProjectRoot
$nodeVersion = Get-NodeVersion -ProjectRoot $projectRoot
$managedRoot = Get-ManagedNodeRoot
$nodeDir = Ensure-ManagedNode -NodeVersion $nodeVersion -ManagedRoot $managedRoot

$nodeExe = Join-Path $nodeDir "node.exe"
$npmCmd = Join-Path $nodeDir "npm.cmd"
$corepackCmd = Join-Path $nodeDir "corepack.cmd"

if (-not (Test-Path $corepackCmd)) {
  throw "corepack.cmd not found in managed Node.js directory: $corepackCmd"
}

if (-not (Test-Path $npmCmd)) {
  throw "npm.cmd not found in managed Node.js directory: $npmCmd"
}

$env:PATH = "$nodeDir;$env:PATH"

Push-Location $projectRoot
try {
  Write-Host "Using managed Node.js from $nodeDir"
  Invoke-ManagedCommand -FilePath $nodeExe -Arguments @("-v") -WorkingDirectory $projectRoot
  Invoke-ManagedCommand -FilePath $corepackCmd -Arguments @("pnpm", "--version") -WorkingDirectory $projectRoot

  Write-Host "Installing dependencies with Corepack-managed pnpm"
  Invoke-ManagedCommand -FilePath $corepackCmd -Arguments @("pnpm", "install", "--frozen-lockfile") -WorkingDirectory $projectRoot

  $devArgs = @("run", "dev")
  if ($args.Count -gt 0) {
    $devArgs += "--"
    $devArgs += $args
  } else {
    $devArgs += "--"
    $devArgs += "--no-watch"
  }

  Write-Host "Starting desktop app"
  Invoke-ManagedCommand -FilePath $npmCmd -Arguments $devArgs -WorkingDirectory $projectRoot
}
finally {
  Pop-Location
}
