# Codex Desktop Detection Test
$nameFragments = @("Codex", "OpenAI Codex")
$excludeFragments = @("CLI", "npm")

Write-Host "=== Codex Desktop Detection Test ===" -ForegroundColor Cyan
Write-Host ""

# Registry detection
Write-Host "[1] Registry Detection" -ForegroundColor Yellow
$found = $false
$roots = @(
    @{ Root = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall"; Name = "HKCU" },
    @{ Root = "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall"; Name = "HKLM" },
    @{ Root = "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"; Name = "HKLM/WOW64" }
)
$fragments = $nameFragments | ForEach-Object { $_.ToLower() }
$excludes = $excludeFragments | ForEach-Object { $_.ToLower() }

foreach ($reg in $roots) {
    $path = $reg.Root
    if (-not (Test-Path $path)) { continue }
    $items = Get-ChildItem -Path $path -ErrorAction SilentlyContinue
    foreach ($item in $items) {
        $displayName = $item.GetValue("DisplayName")
        if (-not $displayName) { continue }
        $dnLower = $displayName.ToLower()
        $matched = $false
        foreach ($frag in $fragments) {
            if ($dnLower -match [regex]::Escape($frag)) { $matched = $true; break }
        }
        if (-not $matched) { continue }
        $excluded = $false
        foreach ($ex in $excludes) {
            if ($dnLower -match [regex]::Escape($ex)) { $excluded = $true; break }
        }
        if ($excluded) { continue }
        Write-Host "    FOUND!" -ForegroundColor Green
        Write-Host "    DisplayName:      $displayName"
        Write-Host "    DisplayVersion:   $($item.GetValue('DisplayVersion'))"
        Write-Host "    InstallLocation:  $($item.GetValue('InstallLocation'))"
        Write-Host "    UninstallString:  $($item.GetValue('UninstallString'))"
        Write-Host "    Source:           $($reg.Name)"
        $found = $true
        break
    }
    if ($found) { break }
}
if (-not $found) { Write-Host "    NOT FOUND" -ForegroundColor Red }
Write-Host ""

# AppX detection
Write-Host "[2] AppX/Store Detection" -ForegroundColor Yellow
$pkg = Get-AppxPackage -Name "OpenAI.Codex" -ErrorAction SilentlyContinue
if ($pkg) {
    Write-Host "    FOUND AppX package!" -ForegroundColor Green
    Write-Host "    Name:             $($pkg.Name)"
    Write-Host "    Version:          $($pkg.Version)"
    Write-Host "    InstallLocation:  $($pkg.InstallLocation)"
} else {
    Write-Host "    NOT FOUND AppX package" -ForegroundColor Red
}
Write-Host ""

# Final result
Write-Host "=== Final Result ===" -ForegroundColor Cyan
if ($found -or $pkg) {
    $ver = if ($found) { $item.GetValue('DisplayVersion') } else { $pkg.Version }
    $loc = if ($found) { $item.GetValue('InstallLocation') } else { $pkg.InstallLocation }
    Write-Host "    installed: true" -ForegroundColor Green
    Write-Host "    version:   $ver"
    Write-Host "    path:      $loc"
} else {
    Write-Host "    installed: false" -ForegroundColor Red
}
