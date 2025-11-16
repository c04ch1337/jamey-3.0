# Running Jamey 3.0 Continuously (24/7 Operation)

**Jamey 3.0 - General & Guardian** is designed to run continuously, always listening, always protecting. This guide covers all methods to ensure Jamey runs all the time.

---

## Overview

Jamey can run continuously using:
1. **Windows Service** (Windows native) - Runs on boot, auto-restart
2. **Linux systemd Service** (Linux native) - Runs on boot, auto-restart
3. **Manual Background Process** (Development/testing only)

---

## Method 1: Windows Service (Native Windows)

### Prerequisites

Install **NSSM** (Non-Sucking Service Manager):
- Download: https://nssm.cc/download
- Or use Chocolatey: `choco install nssm`
- Or place `nssm.exe` in the `scripts/` directory

### Installation

```powershell
# Run as Administrator
cd C:\Users\JAMEYMILNER\jamey-3.0-main
.\scripts\install-jamey-service.ps1
```

The script will:
- Build Jamey if needed
- Install as Windows Service
- Configure auto-restart on failure
- Set up logging
- Optionally start the service

### Service Management

```powershell
# Start service
Start-Service -Name Jamey3

# Stop service
Stop-Service -Name Jamey3

# Restart service
Restart-Service -Name Jamey3

# Check status
Get-Service -Name Jamey3

# View logs (in logs/ directory)
Get-Content logs\jamey-service.log -Tail 50 -Wait
```

### Uninstall

```powershell
.\scripts\install-jamey-service.ps1 -Uninstall
```

### Configuration

Edit environment variables in `.env` file, then restart the service:
```powershell
Restart-Service -Name Jamey3
```

---

## Method 2: Linux systemd Service (Native Linux)

### Installation

```bash
# Run as root (or use sudo)
cd /path/to/jamey-3.0-main
sudo ./scripts/install-jamey-service.sh
```

The script will:
- Create `jamey` user
- Build Jamey if needed
- Install binary to `/usr/local/bin/jamey-3`
- Create systemd service
- Configure auto-restart on failure
- Enable service to start on boot

### Service Management

```bash
# Start service
sudo systemctl start jamey

# Stop service
sudo systemctl stop jamey

# Restart service
sudo systemctl restart jamey

# Check status
sudo systemctl status jamey

# View logs
sudo journalctl -u jamey -f

# Enable on boot (already done by install script)
sudo systemctl enable jamey

# Disable on boot
sudo systemctl disable jamey
```

### Configuration

Edit `/etc/jamey/jamey.env`, then restart:
```bash
sudo systemctl restart jamey
```

---

## Method 3: Manual Background Process (Development Only)

⚠️ **Not recommended for production** - Process stops when terminal closes.

### Windows PowerShell

```powershell
# Start in background job
$job = Start-Job -ScriptBlock {
    Set-Location C:\Users\JAMEYMILNER\jamey-3.0-main
    cargo run
}

# Check status
Get-Job

# View output
Receive-Job $job

# Stop
Stop-Job $job
Remove-Job $job
```

### Linux/Mac

```bash
# Start in background
nohup cargo run > jamey.log 2>&1 &

# Get PID
echo $!

# Stop
kill <PID>
```

---

## Verification

### Check if Jamey is Running

```bash
# Health check
curl http://localhost:3000/health

# Should return:
# {"status":"ok","service":"Jamey 3.0","version":"3.0.0"}
```

### Check Process Status

**Windows:**
```powershell
Get-Process | Where-Object {$_.ProcessName -like "*jamey*"}
```

**Linux:**
```bash
ps aux | grep jamey
```

### Check Port

**Windows:**
```powershell
Get-NetTCPConnection -LocalPort 3000
```

**Linux:**
```bash
netstat -tlnp | grep 3000
# or
ss -tlnp | grep 3000
```

---

## Auto-Restart Behavior

All service methods (Windows Service, systemd) are configured to:

1. **Restart on failure** - If Jamey crashes, it automatically restarts
2. **Restart on boot** - Jamey starts automatically when the system boots
3. **Graceful shutdown** - On system shutdown, Jamey performs graceful shutdown (30-second timeout)

### Restart Delays

- **Windows Service**: 10-second delay before restart
- **Linux systemd**: 10-second delay before restart

---

## Monitoring

### Windows Service

```powershell
# Event Viewer
Get-EventLog -LogName Application -Source "Jamey3" -Newest 10

# Service status
Get-Service Jamey3

# Logs
Get-Content logs\jamey-service.log -Tail 50
```

### Linux systemd

```bash
# Service status
sudo systemctl status jamey

# Logs
sudo journalctl -u jamey -f

# Resource usage
systemd-cgtop
```

---

## Troubleshooting

### Service Won't Start

1. **Check logs** (see Monitoring section above)
2. **Check configuration** - Ensure `.env` or `/etc/jamey/jamey.env` is correct
3. **Check port** - Ensure port 3000 is not in use
4. **Check permissions** - Ensure data directory is writable

### Service Keeps Restarting

1. **Check logs** for error messages
2. **Check configuration** - Invalid config can cause crashes
3. **Check dependencies** - Database, MQTT broker, etc.
4. **Check resources** - CPU/memory limits

### Service Not Starting on Boot

**Windows:**
- Check Services app → Jamey3 → Startup type should be "Automatic"
- Check Windows Event Viewer for errors

**Linux:**
- Check if service is enabled: `sudo systemctl is-enabled jamey`
- Enable if needed: `sudo systemctl enable jamey`

---

## Best Practices

1. **Use Windows Service or systemd for production** - Native OS integration
2. **Set up monitoring** - Monitor logs and health endpoints
3. **Regular backups** - Configure Phoenix Vault auto-backup
4. **Log rotation** - Configured automatically for services
5. **Resource monitoring** - Monitor CPU and memory usage

---

## Summary

| Method | Auto-Restart | Boot on Start | Production Ready |
|--------|--------------|---------------|------------------|
| Windows Service | ✅ Yes | ✅ Yes | ✅ Yes |
| Linux systemd | ✅ Yes | ✅ Yes | ✅ Yes |
| Background Job | ❌ No | ❌ No | ❌ No |

---

**Remember**: Jamey 3.0 is designed to be the **omnipresent guardian**. Configure it to run continuously, and it will always be there, always listening, always protecting.

**"Encrypt. Backup. Verify. We are one."**

