param([switch]$Extra)

$ErrorActionPreference = 'Continue'
Set-Location (Split-Path -Parent $PSScriptRoot)

function Assert-Success([string]$Step) {
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $Step"
        exit $LASTEXITCODE
    }
}

cargo fmt --all -- --check
Assert-Success 'format'

cargo clippy --workspace --all-targets --locked -- -D warnings
Assert-Success 'clippy'

cargo test --workspace --locked
Assert-Success 'test'

if ($Extra) {
    if (Get-Command cargo-deny -ErrorAction SilentlyContinue) {
        cargo deny check
        Assert-Success 'deny'
    } else {
        Write-Host 'skipped: cargo-deny not installed'
    }
    if (Get-Command typos -ErrorAction SilentlyContinue) {
        typos
        Assert-Success 'typos'
    } else {
        Write-Host 'skipped: typos not installed'
    }
    if (Get-Command zizmor -ErrorAction SilentlyContinue) {
        zizmor --offline .
        Assert-Success 'zizmor'
    } else {
        Write-Host 'skipped: zizmor not installed'
    }
}

exit 0
