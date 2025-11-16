# PowerShell script to install Jamey 3.0 as a Windows Service
# Uses NSSM (Non-Sucking Service Manager) - download from https://nssm.cc/download

param(
    [switch]$Uninstall,
    [string]$ServiceName = "Jamey3",
    [string]$DisplayName = "Jamey 3.0 - General & Guardian"
)

$ErrorActionPreference = "Stop"

# Get script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

# Check for NSSM
$nssmPath = Get-Command nssm -ErrorAction SilentlyContinue
if (-not $nssmPath) {
    Write-Host "❌ NSSM (Non-Sucking Service Manager) not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install NSSM:" -ForegroundColor Yellow
    Write-Host "  1. Download from: https://nssm.cc/download" -ForegroundColor Cyan
    Write-Host "  2. Extract and add to PATH, or place nssm.exe in this directory" -ForegroundColor Cyan
    Write-Host "  3. Or use: choco install nssm" -ForegroundColor Cyan
    Write-Host ""
    exit 1
}

$nssmExe = $nssmPath.Source

if ($Uninstall) {
    Write-Host "🗑️  Uninstalling Jamey Windows Service..." -ForegroundColor Yellow
    
    # Stop and remove service
    & $nssmExe stop $ServiceName
    & $nssmExe remove $ServiceName confirm
    
    Write-Host "✅ Service uninstalled" -ForegroundColor Green
    exit 0
}

Write-Host "🚀 Installing Jamey 3.0 as Windows Service..." -ForegroundColor Cyan
Write-Host ""

# Check if service already exists
$existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($existingService) {
    Write-Host "⚠️  Service '$ServiceName' already exists!" -ForegroundColor Yellow
    $response = Read-Host "Remove existing service? (y/n)"
    if ($response -eq 'y') {
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        & $nssmExe remove $ServiceName confirm
        Start-Sleep -Seconds 2
    } else {
        Write-Host "❌ Installation cancelled" -ForegroundColor Red
        exit 1
    }
}

# Build the application if needed
$binaryPath = Join-Path $ProjectRoot "target\release\jamey-3.exe"
if (-not (Test-Path $binaryPath)) {
    Write-Host "📦 Building Jamey 3.0..." -ForegroundColor Yellow
    Push-Location $ProjectRoot
    cargo build --release --bin jamey-3
    Pop-Location
    
    if (-not (Test-Path $binaryPath)) {
        Write-Host "❌ Build failed! Binary not found at: $binaryPath" -ForegroundColor Red
        exit 1
    }
}

# Get absolute path
$binaryPath = Resolve-Path $binaryPath

# Create data directory if it doesn't exist
$dataDir = Join-Path $ProjectRoot "data"
if (-not (Test-Path $dataDir)) {
    New-Item -ItemType Directory -Path $dataDir -Force | Out-Null
}

Write-Host "📝 Configuring service..." -ForegroundColor Yellow

# Install service
& $nssmExe install $ServiceName $binaryPath

# Configure service
& $nssmExe set $ServiceName DisplayName $DisplayName
& $nssmExe set $ServiceName Description "Jamey 3.0 - General & Guardian - Eternal Hive General and Omnipresent Guardian"

# Set working directory
& $nssmExe set $ServiceName AppDirectory $ProjectRoot

# Configure restart behavior (always restart on failure)
& $nssmExe set $ServiceName AppRestartDelay 10000
& $nssmExe set $ServiceName AppExit Default Restart
& $nssmExe set $ServiceName AppThrottle 1500

# Set output files
$logDir = Join-Path $ProjectRoot "logs"
if (-not (Test-Path $logDir)) {
    New-Item -ItemType Directory -Path $logDir -Force | Out-Null
}
& $nssmExe set $ServiceName AppStdout (Join-Path $logDir "jamey-service.log")
& $nssmExe set $ServiceName AppStderr (Join-Path $logDir "jamey-service-error.log")
& $nssmExe set $ServiceName AppRotateFiles 1
& $nssmExe set $ServiceName AppRotateOnline 1
& $nssmExe set $ServiceName AppRotateSeconds 86400
& $nssmExe set $ServiceName AppRotateBytes 10485760

# Set environment variables (if .env file exists, load it)
$envFile = Join-Path $ProjectRoot ".env"
if (Test-Path $envFile) {
    Write-Host "📋 Loading environment variables from .env..." -ForegroundColor Yellow
    Get-Content $envFile | ForEach-Object {
        if ($_ -match '^\s*([^#][^=]+)=(.*)$') {
            $key = $matches[1].Trim()
            $value = $matches[2].Trim()
            & $nssmExe set $ServiceName AppEnvironmentExtra "$key=$value"
        }
    }
}

# Set startup type to automatic (start on boot)
& $nssmExe set $ServiceName Start SERVICE_AUTO_START

Write-Host ""
Write-Host "✅ Service installed successfully!" -ForegroundColor Green
Write-Host ""

# Ask if user wants to start now
$response = Read-Host "Start service now? (y/n)"
if ($response -eq 'y') {
    Write-Host "🚀 Starting service..." -ForegroundColor Yellow
    Start-Service -Name $ServiceName
    Start-Sleep -Seconds 3
    
    $service = Get-Service -Name $ServiceName
    if ($service.Status -eq 'Running') {
        Write-Host "✅ Service is running!" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Service started but status is: $($service.Status)" -ForegroundColor Yellow
        Write-Host "   Check logs at: $logDir" -ForegroundColor Cyan
    }
} else {
    Write-Host "ℹ️  Service installed but not started" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "Service management commands:" -ForegroundColor Cyan
Write-Host "  Start:   Start-Service -Name $ServiceName" -ForegroundColor White
Write-Host "  Stop:    Stop-Service -Name $ServiceName" -ForegroundColor White
Write-Host "  Status:  Get-Service -Name $ServiceName" -ForegroundColor White
Write-Host "  Restart: Restart-Service -Name $ServiceName" -ForegroundColor White
Write-Host ""
Write-Host "Logs location: $logDir" -ForegroundColor Cyan
Write-Host ""

