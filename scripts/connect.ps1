# PowerShell version of connect.sh for Windows
# SSH-like interface: Connect to a running Jamey 3.0 backend

$BACKEND_URL = if ($env:JAMEY_API_URL) { $env:JAMEY_API_URL } else { "http://localhost:3000" }
$API_KEY = if ($env:JAMEY_API_KEY) { $env:JAMEY_API_KEY } else { $null }

Write-Host "🔌 Connecting to Jamey 3.0 Backend..." -ForegroundColor Cyan
Write-Host "   URL: $BACKEND_URL" -ForegroundColor Gray
Write-Host ""

# Check if backend is running
try {
    $response = Invoke-WebRequest -Uri "$BACKEND_URL/health" -TimeoutSec 2 -ErrorAction Stop
    Write-Host "✅ Connected to backend!" -ForegroundColor Green
} catch {
    Write-Host "❌ Backend is not running at $BACKEND_URL" -ForegroundColor Red
    Write-Host ""
    Write-Host "💡 To start the backend:" -ForegroundColor Yellow
    Write-Host "   .\scripts\run.ps1" -ForegroundColor Gray
    Write-Host "   or" -ForegroundColor Gray
    Write-Host "   cargo run" -ForegroundColor Gray
    Write-Host ""
    exit 1
}

Write-Host ""
Write-Host "💬 Starting interactive chat..." -ForegroundColor Cyan
Write-Host "   (Type /help for commands, /exit to disconnect)" -ForegroundColor Gray
Write-Host ""

# Use the CLI in connect mode
if ($API_KEY) {
    cargo run --bin jamey-cli connect --url $BACKEND_URL --api-key $API_KEY
} else {
    cargo run --bin jamey-cli connect --url $BACKEND_URL
}

