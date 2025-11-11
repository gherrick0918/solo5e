#!/usr/bin/env pwsh
# CI-style checks: format, lint, test
# Usage: .\check.ps1

Write-Host "Running fmt..." -ForegroundColor Cyan
cargo fmt --all
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Running clippy..." -ForegroundColor Cyan
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Running tests..." -ForegroundColor Cyan
cargo test --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "All checks passed!" -ForegroundColor Green
