# CLI Tools Detection Test
# Tests if tools are on PATH and can report version

$tools = @(
    @{ Name = "Claude Code CLI"; Command = "claude"; Expected = "unknown" },
    @{ Name = "Codex CLI"; Command = "codex"; Expected = "unknown" },
    @{ Name = "Gemini CLI"; Command = "gemini"; Expected = "unknown" },
    @{ Name = "OpenCode CLI"; Command = "opencode"; Expected = "unknown" },
    @{ Name = "OpenClaw"; Command = "openclaw"; Expected = "unknown" },
    @{ Name = "Hermes"; Command = "hermes"; Expected = "unknown" },
    @{ Name = "Node.js"; Command = "node"; Expected = "unknown" },
    @{ Name = "Git"; Command = "git"; Expected = "unknown" }
)

Write-Host "=== CLI Tools Detection Test ===" -ForegroundColor Cyan
Write-Host ""

foreach ($tool in $tools) {
    Write-Host "[$($tool.Name)]" -ForegroundColor Yellow

    $found = $false
    $version = $null

    # Try direct command
    try {
        $output = & $tool.Command --version 2>&1
        if ($LASTEXITCODE -eq 0 -and $output) {
            $version = ($output | Out-String).Trim()
            if ($version -and $version.Length -gt 0) {
                $found = $true
            }
        }
    } catch {
        # Not found
    }

    if ($found) {
        Write-Host "    FOUND: $version" -ForegroundColor Green
    } else {
        Write-Host "    NOT FOUND on PATH" -ForegroundColor Red
    }
    Write-Host ""
}
