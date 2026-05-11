Set-StrictMode -Version Latest

function Test-IsLocalManagedNodeDir {
  param(
    [Parameter(Mandatory = $true)]
    [string]$ProjectRoot,
    [Parameter(Mandatory = $true)]
    [string]$ManagedNodeDir
  )

  return $ManagedNodeDir.StartsWith(
    $ProjectRoot,
    [System.StringComparison]::OrdinalIgnoreCase
  )
}

function Test-IsProjectRendererDevServer {
  param(
    [Parameter(Mandatory = $true)]
    [string]$ProjectRoot,
    [Parameter(Mandatory = $true)]
    [string]$ManagedNodeDir,
    [Parameter(Mandatory = $true)]
    [string]$ExecutablePath,
    [Parameter(Mandatory = $true)]
    [string]$CommandLine
  )

  $isManagedNodeProcess = $ExecutablePath.StartsWith(
    $ManagedNodeDir,
    [System.StringComparison]::OrdinalIgnoreCase
  )
  if (-not $isManagedNodeProcess) {
    return $false
  }

  if (-not $CommandLine.Contains("node_modules/vite/bin/vite.js")) {
    return $false
  }

  # Repo-local managed Node launches Vite with a relative command line, so the
  # project root may never appear in the process arguments on Windows.
  if (Test-IsLocalManagedNodeDir -ProjectRoot $ProjectRoot -ManagedNodeDir $ManagedNodeDir) {
    return $true
  }

  return $CommandLine.Contains($ProjectRoot)
}
