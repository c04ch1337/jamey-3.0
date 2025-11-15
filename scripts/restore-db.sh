#!/bin/bash
set -euo pipefail

# Configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BACKUP_DIR="${PROJECT_ROOT}/data/backups"
DB_PATH="${PROJECT_ROOT}/data/jamey.db"
MEMORY_DIR="${PROJECT_ROOT}/data/memory"

# Function to log with timestamp
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

# Function to list available backups
list_backups() {
    log "Available backups:"
    for backup in "$BACKUP_DIR"/jamey_backup_*; do
        if [ -d "$backup" ]; then
            local manifest="${backup}/manifest.json"
            if [ -f "$manifest" ]; then
                local timestamp=$(jq -r '.timestamp' "$manifest")
                local db_size=$(jq -r '.database_size' "$manifest")
                echo "  $(basename "$backup") - $timestamp (DB size: $db_size bytes)"
            else
                echo "  $(basename "$backup") (no manifest)"
            fi
        fi
    done
}

# Show usage if no arguments provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <backup-name>"
    echo "Example: $0 jamey_backup_20251115_123456"
    echo
    list_backups
    exit 1
fi

BACKUP_NAME="$1"
BACKUP_PATH="${BACKUP_DIR}/${BACKUP_NAME}"

# Check if backup exists
if [ ! -d "$BACKUP_PATH" ]; then
    log "Error: Backup not found at $BACKUP_PATH"
    echo
    list_backups
    exit 1
fi

# Verify backup integrity
log "Verifying backup integrity..."
cd "$BACKUP_PATH"
if ! shasum -a 256 -c SHA256SUMS; then
    log "Error: Backup verification failed"
    exit 1
fi

# Stop the Jamey service if it's running
if systemctl is-active --quiet jamey; then
    log "Stopping Jamey service..."
    sudo systemctl stop jamey
fi

# Create backup of current data
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
log "Creating backup of current data..."
if [ -f "$DB_PATH" ]; then
    cp "$DB_PATH" "${DB_PATH}.${TIMESTAMP}.bak"
fi
if [ -d "$MEMORY_DIR" ]; then
    tar -czf "${MEMORY_DIR}.${TIMESTAMP}.tar.gz" -C "$(dirname "$MEMORY_DIR")" "$(basename "$MEMORY_DIR")"
fi

# Restore database
log "Restoring SQLite database..."
mkdir -p "$(dirname "$DB_PATH")"
cp "${BACKUP_PATH}/jamey.db" "$DB_PATH"
chmod 600 "$DB_PATH"

# Restore memory indices
if [ -f "${BACKUP_PATH}/memory.tar.gz" ]; then
    log "Restoring memory indices..."
    rm -rf "$MEMORY_DIR"
    tar -xzf "${BACKUP_PATH}/memory.tar.gz" -C "$(dirname "$MEMORY_DIR")"
    chmod -R 700 "$MEMORY_DIR"
fi

# Verify restored database
log "Verifying restored database..."
if sqlite3 "$DB_PATH" "PRAGMA integrity_check;" | grep -q "ok"; then
    log "Database integrity check passed"
else
    log "Error: Database integrity check failed"
    log "Rolling back to backup..."
    mv "${DB_PATH}.${TIMESTAMP}.bak" "$DB_PATH"
    if [ -f "${MEMORY_DIR}.${TIMESTAMP}.tar.gz" ]; then
        rm -rf "$MEMORY_DIR"
        tar -xzf "${MEMORY_DIR}.${TIMESTAMP}.tar.gz" -C "$(dirname "$MEMORY_DIR")"
    fi
    exit 1
fi

# Start the Jamey service if it was running
if systemctl is-active --quiet jamey; then
    log "Starting Jamey service..."
    sudo systemctl start jamey
fi

log "Restore completed successfully"
log "Previous data backed up with timestamp: ${TIMESTAMP}"