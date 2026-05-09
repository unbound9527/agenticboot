# OpenCode Desktop Detection Test
$nameFragments = @("OpenCode")
$excludeFragments = @("CLI", "npm")

Write-Host "=== OpenCode Desktop Detection Test ===" -ForegroundColor Cyan
Write-Host ""

$roots = @(
    @{ Root = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall"; Name = "HKCU" },
    @{ Root = "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall"; Name = "HKLM" },
    @{ Root = "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"; Name = "HKLM/WOW64" }
)
$fragments = $nameFragments | ForEach-Object { $_.ToLower() }
$excludes = $excludeFragments | ForEach-Object { $_.ToLower() }

$found = $false
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

        $installLoc = $item.GetValue("InstallLocation")
        $displayIcon = $item.GetValue("DisplayIcon")

        # Simulate Rust path logic: install_location.or(display_icon.parent())
        $resolvedPath = $installLoc
        if (-not $resolvedPath -and $displayIcon) {
            $iconPath = ($displayIcon -split ',')[0].Trim().Trim('"')
            $resolvedPath = Split-Path $iconPath -Parent
        }

        Write-Host "[1] Registry Detection" -ForegroundColor Yellow
        Write-Host "    FOUND!" -ForegroundColor Green
        Write-Host "    DisplayName:      $displayName"
        Write-Host "    DisplayVersion:   $($item.GetValue('DisplayVersion'))"
        Write-Host "    InstallLocation:  $installLoc"
        Write-Host "    DisplayIcon:      $displayIcon"
        Write-Host "    ResolvedPath:     $resolvedPath"
        Write-Host "    UninstallString:  $($item.GetValue('UninstallString'))"
        Write-Host "    Source:           $($reg.Name)"

        $found = $true
        break
    }
    if ($found) { break }
}
if (-not $found) { Write-Host "[1] Registry Detection: NOT FOUND" -ForegroundColor Red }
Write-Host ""

# AppX detection
Write-Host "[2] AppX/Store Detection" -ForegroundColor Yellow
$pkg = Get-AppxPackage -Name "OpenCode" -ErrorAction SilentlyContinue
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
if ($found) {
    Write-Host "    installed: true (registry)" -ForegroundColor Green
    Write-Host "    version:   $($item.GetValue('DisplayVersion'))"
    Write-Host "    path:      $resolvedPath"
} elseif ($pkg) {
    Write-Host "    installed: true (AppX)" -ForegroundColor Green
    Write-Host "    version:   $($pkg.Version)"
    Write-Host "    path:      $($pkg.InstallLocation)"
} else {
    Write-Host "    installed: false" -ForegroundColor Red
}
