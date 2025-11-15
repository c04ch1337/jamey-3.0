#!/bin/bash
set -euo pipefail

# Configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BACKUP_DIR="${PROJECT_ROOT}/data/backups"
DB_PATH="${PROJECT_ROOT}/data/jamey.db"
MEMORY_DIR="${PROJECT_ROOT}/data/memory"
MAX_BACKUPS=10

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Generate timestamp for backup files
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="jamey_backup_${TIMESTAMP}"
BACKUP_PATH="${BACKUP_DIR}/${BACKUP_NAME}"

# Function to log with timestamp
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

# Function to cleanup old backups
cleanup_old_backups() {
    local count=$(ls -1 "$BACKUP_DIR" | wc -l)
    if [ "$count" -gt "$MAX_BACKUPS" ]; then
        log "Cleaning up old backups (keeping last $MAX_BACKUPS)..."
        ls -1t "$BACKUP_DIR" | tail -n +$((MAX_BACKUPS + 1)) | while read backup; do
            log "Removing old backup: $backup"
            rm -rf "${BACKUP_DIR}/${backup}"
        done
    fi
}

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
    log "Error: Database file not found at $DB_PATH"
    exit 1
fi

# Create backup directory for this run
mkdir -p "$BACKUP_PATH"

# Backup SQLite database
log "Backing up SQLite database..."
sqlite3 "$DB_PATH" ".backup '${BACKUP_PATH}/jamey.db'"

# Backup memory indices
log "Backing up memory indices..."
if [ -d "$MEMORY_DIR" ]; then
    tar -czf "${BACKUP_PATH}/memory.tar.gz" -C "$(dirname "$MEMORY_DIR")" "$(basename "$MEMORY_DIR")"
else
    log "Warning: Memory directory not found at $MEMORY_DIR"
fi

# Create backup manifest
cat > "${BACKUP_PATH}/manifest.json" << EOF
{
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "backup_name": "${BACKUP_NAME}",
    "components": {
        "database": "jamey.db",
        "memory": "memory.tar.gz"
    },
    "database_size": "$(stat -f%z "${BACKUP_PATH}/jamey.db")",
    "memory_size": "$(stat -f%z "${BACKUP_PATH}/memory.tar.gz" 2>/dev/null || echo "0")"
}
EOF

# Create checksum file
log "Generating checksums..."
cd "$BACKUP_PATH"
shasum -a 256 * > SHA256SUMS

# Cleanup old backups
cleanup_old_backups

# Print backup info
log "Backup completed successfully:"
log "  Location: $BACKUP_PATH"
log "  Database size: $(stat -f%z "${BACKUP_PATH}/jamey.db") bytes"
if [ -f "${BACKUP_PATH}/memory.tar.gz" ]; then
    log "  Memory indices size: $(stat -f%z "${BACKUP_PATH}/memory.tar.gz") bytes"
fi

# Verify backup
log "Verifying backup integrity..."
cd "$BACKUP_PATH"
if shasum -a 256 -c SHA256SUMS; then
    log "Backup verification successful"
else
    log "Error: Backup verification failed"
    exit 1
fi