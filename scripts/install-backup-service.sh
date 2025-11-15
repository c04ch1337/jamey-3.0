#!/bin/bash
set -euo pipefail

# Configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Function to log with timestamp
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    log "Error: This script must be run as root"
    exit 1
fi

# Create jamey user if it doesn't exist
if ! id -u jamey &>/dev/null; then
    log "Creating jamey user..."
    useradd -r -s /bin/false jamey
fi

# Create necessary directories
log "Creating directories..."
install -d -m 755 /opt/jamey
install -d -m 700 -o jamey -g jamey /opt/jamey/data
install -d -m 700 -o jamey -g jamey /opt/jamey/data/backups

# Copy scripts
log "Installing backup scripts..."
install -m 755 -o root -g root "${SCRIPT_DIR}/backup-db.sh" /opt/jamey/scripts/
install -m 755 -o root -g root "${SCRIPT_DIR}/restore-db.sh" /opt/jamey/scripts/

# Install systemd service and timer
log "Installing systemd service and timer..."
install -m 644 -o root -g root "${SCRIPT_DIR}/jamey-backup.service" /etc/systemd/system/
install -m 644 -o root -g root "${SCRIPT_DIR}/jamey-backup.timer" /etc/systemd/system/

# Reload systemd
log "Reloading systemd..."
systemctl daemon-reload

# Enable and start timer
log "Enabling and starting backup timer..."
systemctl enable jamey-backup.timer
systemctl start jamey-backup.timer

# Show status
log "Installation complete. Service status:"
systemctl status jamey-backup.timer

log "Next scheduled backup time:"
systemctl list-timers jamey-backup.timer

log "
Installation complete! The backup service will run daily at 2 AM.

Manual operations:
- Run backup now:    sudo systemctl start jamey-backup.service
- View backup logs:  sudo journalctl -u jamey-backup.service
- List backups:      /opt/jamey/scripts/restore-db.sh
- Restore backup:    sudo /opt/jamey/scripts/restore-db.sh <backup-name>
"