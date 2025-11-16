#!/bin/bash
set -euo pipefail

# Install Jamey 3.0 as a systemd service for continuous operation
# This script sets up Jamey to run automatically on boot and restart on failure

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Function to log with timestamp
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    log "Error: This script must be run as root (use sudo)"
    exit 1
fi

log "Installing Jamey 3.0 as a systemd service..."

# Create jamey user if it doesn't exist
if ! id -u jamey &>/dev/null; then
    log "Creating jamey user..."
    useradd -r -s /bin/false -d /opt/jamey jamey
fi

# Create necessary directories
log "Creating directories..."
install -d -m 755 /opt/jamey
install -d -m 755 /opt/jamey/data
install -d -m 755 /opt/jamey/data/backups
install -d -m 755 /opt/jamey/data/memory
install -d -m 755 /etc/jamey

# Build the application if binary doesn't exist
if [ ! -f "${PROJECT_ROOT}/target/release/jamey-3" ]; then
    log "Building Jamey 3.0..."
    cd "$PROJECT_ROOT"
    cargo build --release --bin jamey-3
fi

# Copy binary
log "Installing binary..."
install -m 755 -o root -g root "${PROJECT_ROOT}/target/release/jamey-3" /usr/local/bin/jamey-3

# Copy configuration template if it doesn't exist
if [ ! -f /etc/jamey/jamey.env ]; then
    log "Creating environment file template..."
    cat > /etc/jamey/jamey.env <<EOF
# Jamey 3.0 Environment Configuration
# Edit this file and set your configuration values

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# OpenRouter API (required for LLM features)
OPENROUTER_API_KEY=
OPENROUTER_MODEL=kimi-k2-thinking
OPENROUTER_API_URL=https://openrouter.ai/api/v1

# Data directory
DATA_DIR=/opt/jamey/data

# Logging
RUST_LOG=info

# Phoenix Vault (optional)
PHOENIX_ENABLED=false
PHOENIX_ENCRYPTION_KEY=
PHOENIX_BACKUP_DIR=/opt/jamey/data/backups
PHOENIX_AUTO_BACKUP_HOURS=24
PHOENIX_MAX_BACKUPS=10

# MQTT (optional)
MQTT_ENABLED=false
MQTT_BROKER_URL=
MQTT_CLIENT_ID=jamey-3
MQTT_USERNAME=
MQTT_PASSWORD=
EOF
    chmod 600 /etc/jamey/jamey.env
    log "⚠️  Please edit /etc/jamey/jamey.env and set your configuration values"
fi

# Copy migrations
log "Copying migrations..."
if [ -d "${PROJECT_ROOT}/migrations" ]; then
    cp -r "${PROJECT_ROOT}/migrations" /opt/jamey/
    chown -R jamey:jamey /opt/jamey/migrations
fi

# Install systemd service
log "Installing systemd service..."
install -m 644 -o root -g root "${SCRIPT_DIR}/jamey.service" /etc/systemd/system/

# Set ownership of data directory
chown -R jamey:jamey /opt/jamey/data

# Reload systemd
log "Reloading systemd..."
systemctl daemon-reload

# Enable service (start on boot)
log "Enabling Jamey service (will start on boot)..."
systemctl enable jamey.service

# Ask if user wants to start now
read -p "Start Jamey service now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    log "Starting Jamey service..."
    systemctl start jamey.service
    sleep 2
    systemctl status jamey.service --no-pager
else
    log "Service installed but not started. Start it with: sudo systemctl start jamey"
fi

log ""
log "✅ Installation complete!"
log ""
log "Service management commands:"
log "  Start:   sudo systemctl start jamey"
log "  Stop:    sudo systemctl stop jamey"
log "  Status:  sudo systemctl status jamey"
log "  Logs:    sudo journalctl -u jamey -f"
log "  Restart: sudo systemctl restart jamey"
log ""
log "Configuration:"
log "  Edit:    sudo nano /etc/jamey/jamey.env"
log "  Then:    sudo systemctl restart jamey"
log ""

