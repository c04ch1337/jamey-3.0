# PowerShell script to start the backend for development

$ErrorActionPreference = "Stop"

Write-Host "Starting Jamey 3.0 Backend..." -ForegroundColor Cyan
Write-Host ""

# Get project root
$ProjectRoot = Split-Path -Parent $PSScriptRoot

# Kill any process on port 3000 to prevent conflicts
Write-Host "Checking for existing processes on port 3000..." -ForegroundColor Yellow
$existingProcess = Get-NetTCPConnection -LocalPort 3000 -ErrorAction SilentlyContinue
if ($existingProcess) {
    Stop-Process -Id $existingProcess.OwningProcess -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
    Write-Host "Port 3000 cleared." -ForegroundColor Green
} else {
    Write-Host "Port 3000 is available." -ForegroundColor Green
}

Write-Host ""
Write-Host "Starting Backend..." -ForegroundColor Cyan
Write-Host "   Backend will run on: http://localhost:3000" -ForegroundColor Gray
Write-Host ""

# Start backend in new window
$backendScript = @"
cd '$ProjectRoot'
Write-Host 'Jamey 3.0 Backend Starting...' -ForegroundColor Cyan
Write-Host '   Press Ctrl+C to stop' -ForegroundColor Gray
Write-Host ''
cargo run
"@

Start-Process powershell -ArgumentList "-NoExit", "-Command", $backendScript

Write-Host "Waiting for backend to start..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

# Wait for backend to be ready
$maxAttempts = 30
$attempt = 0
$ready = $false

while ($attempt -lt $maxAttempts) {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:3000/health" -TimeoutSec 2 -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            Write-Host "Backend is ready!" -ForegroundColor Green
            $ready = $true
            break
        }
    } catch {
        # Backend not ready yet
    }
    Start-Sleep -Seconds 1
    $attempt++
}

if (-not $ready) {
    Write-Host "Backend may still be starting..." -ForegroundColor Yellow
    Write-Host "   Check the backend window for status" -ForegroundColor Gray
}

Write-Host ""
Write-Host "Backend started!" -ForegroundColor Green
Write-Host ""
Write-Host "Quick Reference:" -ForegroundColor Cyan
Write-Host "   Backend:  http://localhost:3000" -ForegroundColor White
Write-Host "   Health:   http://localhost:3000/health" -ForegroundColor White
Write-Host ""
Write-Host "Tips:" -ForegroundColor Cyan
Write-Host "   - Backend changes require restart (Ctrl+C in backend window, then cargo run)" -ForegroundColor Gray
Write-Host "   - Close the window to stop the backend" -ForegroundColor Gray
Write-Host ""
