Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "dev-desktop.helpers.ps1")

function Assert-True {
  param(
    [Parameter(Mandatory = $true)]
    [bool]$Condition,
    [Parameter(Mandatory = $true)]
    [string]$Message
  )

  if (-not $Condition) {
    throw $Message
  }
}

function Assert-False {
  param(
    [Parameter(Mandatory = $true)]
    [bool]$Condition,
    [Parameter(Mandatory = $true)]
    [string]$Message
  )

  if ($Condition) {
    throw $Message
  }
}

$projectRoot = "D:\projects\agenticboot"
$localManagedNodeDir = Join-Path $projectRoot ".managed-node\node-v24.15.0-win-x64"
$localRenderer = [pscustomobject]@{
  ExecutablePath = Join-Path $localManagedNodeDir "node.exe"
  CommandLine    = 'node  node_modules/vite/bin/vite.js'
}

Assert-True `
  -Condition (Test-IsProjectRendererDevServer `
    -ProjectRoot $projectRoot `
    -ManagedNodeDir $localManagedNodeDir `
    -ExecutablePath $localRenderer.ExecutablePath `
    -CommandLine $localRenderer.CommandLine) `
  -Message "Expected repo-local managed Node renderer to be detected as stale"

$sharedManagedNodeDir = "D:\shared-managed-node\node-v24.15.0-win-x64"
$sharedRenderer = [pscustomobject]@{
  ExecutablePath = Join-Path $sharedManagedNodeDir "node.exe"
  CommandLine    = 'node  node_modules/vite/bin/vite.js'
}

Assert-False `
  -Condition (Test-IsProjectRendererDevServer `
    -ProjectRoot $projectRoot `
    -ManagedNodeDir $sharedManagedNodeDir `
    -ExecutablePath $sharedRenderer.ExecutablePath `
    -CommandLine $sharedRenderer.CommandLine) `
  -Message "Expected shared managed Node renderer without project root context to be ignored"

Write-Host "dev-desktop helper tests passed"
