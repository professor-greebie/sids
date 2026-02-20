# Code Coverage Script for SIDS Project
# This script runs code coverage and automatically cleans up artifacts
# All operations stay within C:\Studentwork as required by institutional policy

param(
    [switch]$Clean,
    [switch]$Html,
    [switch]$Json,
    [switch]$Lcov,
    [string]$OutputFormat = "html"
)

$ErrorActionPreference = "Stop"

# Ensure we're in the project root
Set-Location $PSScriptRoot

Write-Host "=== SIDS Code Coverage Runner ===" -ForegroundColor Cyan
Write-Host ""

# Function to clean up coverage artifacts
function Clean-CoverageArtifacts {
    Write-Host "Cleaning coverage artifacts..." -ForegroundColor Yellow
    
    $pathsToClean = @(
        "target\coverage",
        "target\tmp\*.profraw",
        "target\*.profraw",
        "target\*.profdata",
        "coverage"
    )
    
    foreach ($path in $pathsToClean) {
        $fullPath = Join-Path $PSScriptRoot $path
        if (Test-Path $fullPath) {
            Write-Host "  Removing: $path" -ForegroundColor Gray
            Remove-Item -Path $fullPath -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
    
    Write-Host "Coverage artifacts cleaned." -ForegroundColor Green
    Write-Host ""
}

# If -Clean flag is provided, clean and exit
if ($Clean) {
    Clean-CoverageArtifacts
    exit 0
}

# Check if cargo-llvm-cov is installed
Write-Host "Checking for cargo-llvm-cov..." -ForegroundColor Yellow
$llvmCovInstalled = $null -ne (Get-Command cargo-llvm-cov -ErrorAction SilentlyContinue)

if (-not $llvmCovInstalled) {
    Write-Host "cargo-llvm-cov not found. Installing..." -ForegroundColor Yellow
    cargo install cargo-llvm-cov
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to install cargo-llvm-cov" -ForegroundColor Red
        exit 1
    }
}

# Clean before running
Clean-CoverageArtifacts

# Create coverage output directory within project
$coverageDir = Join-Path $PSScriptRoot "target\coverage"
New-Item -ItemType Directory -Force -Path $coverageDir | Out-Null

Write-Host "Running code coverage with streaming feature..." -ForegroundColor Yellow
Write-Host ""

# Determine output format
$formatArgs = @()
if ($Html) {
    $formatArgs += "--html"
} elseif ($Json) {
    $formatArgs += "--json"
} elseif ($Lcov) {
    $formatArgs += "--lcov"
} else {
    # Default to HTML
    $formatArgs += "--html"
}

# Run cargo-llvm-cov with proper flags
# --output-dir ensures all outputs stay in project directory
# --features streaming enables the streaming feature for coverage
# --lib runs tests for library code only
cargo llvm-cov `
    --features streaming `
    --lib `
    --output-dir "$coverageDir" `
    @formatArgs

$exitCode = $LASTEXITCODE

Write-Host ""

if ($exitCode -eq 0) {
    Write-Host "=== Coverage Complete ===" -ForegroundColor Green
    Write-Host ""
    Write-Host "Coverage report generated at:" -ForegroundColor Cyan
    
    if ($Html -or ($formatArgs -contains "--html")) {
        $htmlReport = Join-Path $coverageDir "html\index.html"
        if (Test-Path $htmlReport) {
            Write-Host "  HTML: $htmlReport" -ForegroundColor White
            Write-Host ""
            Write-Host "Opening coverage report in browser..." -ForegroundColor Yellow
            Start-Process $htmlReport
        }
    } elseif ($Json) {
        Write-Host "  JSON: $coverageDir\coverage.json" -ForegroundColor White
    } elseif ($Lcov) {
        Write-Host "  LCOV: $coverageDir\lcov.info" -ForegroundColor White
    }
    
    Write-Host ""
    Write-Host "To clean up coverage artifacts, run:" -ForegroundColor Yellow
    Write-Host "  .\run-coverage.ps1 -Clean" -ForegroundColor White
} else {
    Write-Host "=== Coverage Failed ===" -ForegroundColor Red
    Write-Host "Exit code: $exitCode" -ForegroundColor Red
    
    # Clean up on failure
    Write-Host ""
    Clean-CoverageArtifacts
}

exit $exitCode
