# PowerShell version of start-with-cli.sh for Windows
# Start Jamey 3.0 backend and CLI together

Write-Host "🚀 Starting Jamey 3.0 Backend + CLI..." -ForegroundColor Cyan

# Kill any existing server on port 3000
$existing = Get-NetTCPConnection -LocalPort 3000 -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "🛑 Killing existing process on port 3000..." -ForegroundColor Yellow
    $existing | ForEach-Object { 
        Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue 
    }
    Start-Sleep -Seconds 1
    Write-Host "✅ Port 3000 cleared" -ForegroundColor Green
} else {
    Write-Host "✅ Port 3000 is free" -ForegroundColor Green
}

Write-Host ""
Write-Host "📡 Starting backend server..." -ForegroundColor Cyan

# Start backend in background
$backendJob = Start-Job -ScriptBlock {
    Set-Location $using:PWD
    cargo run 2>&1 | Out-File -FilePath "$env:TEMP\jamey-backend.log" -Encoding utf8
}

# Wait for backend to be ready
Write-Host "⏳ Waiting for backend to start..." -ForegroundColor Yellow
$maxAttempts = 30
$attempt = 0
$ready = $false

while ($attempt -lt $maxAttempts) {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:3000/health" -TimeoutSec 1 -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            Write-Host "✅ Backend is ready!" -ForegroundColor Green
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
    Write-Host "❌ Backend failed to start. Check $env:TEMP\jamey-backend.log" -ForegroundColor Red
    Stop-Job $backendJob -ErrorAction SilentlyContinue
    Remove-Job $backendJob -ErrorAction SilentlyContinue
    exit 1
}

Write-Host ""
Write-Host "💬 Starting CLI chat interface..." -ForegroundColor Cyan
Write-Host "   (Backend running in background, Job ID: $($backendJob.Id))" -ForegroundColor Gray
Write-Host "   (Backend logs: $env:TEMP\jamey-backend.log)" -ForegroundColor Gray
Write-Host ""

# Trap to cleanup on exit
$cleanup = {
    Write-Host ""
    Write-Host "🛑 Stopping backend..." -ForegroundColor Yellow
    Stop-Job $backendJob -ErrorAction SilentlyContinue
    Remove-Job $backendJob -ErrorAction SilentlyContinue
}

# Register cleanup on script exit
Register-EngineEvent PowerShell.Exiting -Action $cleanup

try {
    # Start CLI
    cargo run --bin jamey-cli chat
} finally {
    # Cleanup
    & $cleanup
}

