# Database Setup - Complete Guide

## ✅ Automatic Setup

**Good news!** The database is automatically created and initialized when you run the application. No manual setup required!

## What Happens Automatically

When you run `cargo run --release`, the application:

1. ✅ Creates `data/` directory if it doesn't exist
2. ✅ Creates `data/jamey.db` SQLite database file
3. ✅ Runs all migrations from `migrations/` directory
4. ✅ Initializes the 5-layer memory system indices
5. ✅ Starts the server

## Database Location

- **Path**: `data/jamey.db` (relative to where you run the application)
- **Type**: SQLite 3
- **Auto-created**: Yes, on first run

## Migrations

All migrations are automatically applied. Current migrations:

1. **`20241114000000_init.sql`** - Initial schema
   - Creates `app_metadata` table

2. **`20241114200000_soul_system.sql`** - Soul system
   - Creates `soul_entities`, `soul_emotions`, `soul_memory_links` tables

3. **`20241115000000_api_keys.sql`** - API key management (NEW!)
   - Creates `api_keys` table with rotation support
   - Indexes for performance

## Verify Database Setup

### Check Database Exists

```bash
# Linux/Mac
ls -lh data/jamey.db

# Windows PowerShell
Test-Path data\jamey.db
```

### View Tables

```bash
sqlite3 data/jamey.db ".tables"
```

Expected output:
```
api_keys          soul_emotions      soul_memory_links
app_metadata      soul_entities     _sqlx_migrations
```

### Check API Keys Table Schema

```bash
sqlite3 data/jamey.db ".schema api_keys"
```

## Manual Setup (Optional)

If you want to run migrations manually:

### Install sqlx-cli

```bash
cargo install sqlx-cli --no-default-features --features sqlite
```

### Run Migrations

```bash
sqlx migrate run --database-url "sqlite:data/jamey.db"
```

## Create Your First API Key

After the database is set up, create an API key:

### Option 1: Using Helper Script (Linux/Mac)

```bash
chmod +x scripts/create-api-key.sh
./scripts/create-api-key.sh initial-key 60
```

### Option 2: Programmatically (Rust)

```rust
use jamey_3::api::key_manager::ApiKeyManager;
use std::sync::Arc;

let pool = // your database pool
let key_manager = Arc::new(ApiKeyManager::new(Arc::new(pool)));
let key = key_manager.create_key("initial-key", None, Some(60)).await?;
println!("Created API key: {}", key);
```

### Option 3: Direct SQL (Not Recommended)

```sql
-- First, generate a key and hash it
-- Key: jamey_abc123...
-- Hash: sha256(key)

INSERT INTO api_keys (key_hash, name, created_at, rate_limit_per_minute)
VALUES ('<sha256-hash>', 'initial-key', datetime('now'), 60);
```

**Note**: Keys are hashed with SHA-256 before storage, so you must hash the key before inserting.

## Reset Database (If Needed)

If you need to start fresh:

```bash
# Stop the application first (Ctrl+C)
rm data/jamey.db        # Linux/Mac
del data\jamey.db       # Windows CMD
Remove-Item data\jamey.db  # Windows PowerShell

# Then run again - it will recreate automatically
cargo run --release
```

## Troubleshooting

### "unable to open database file"

**Solution**: Install SQLite development libraries:

```bash
# Ubuntu/Debian
sudo apt-get install libsqlite3-dev

# macOS (with Homebrew)
brew install sqlite

# Windows
# SQLite is usually included, but you may need to install Visual C++ Redistributable
```

### Permission Errors

Make sure the `data/` directory is writable:

```bash
chmod 755 data/  # Linux/Mac
```

### Migration Errors

If migrations fail:

1. Check migration files are correct
2. Verify database isn't locked (stop the application)
3. Check SQLite version compatibility

```bash
sqlite3 --version
```

## Database Schema

### api_keys Table

```sql
CREATE TABLE api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TEXT,
    revoked_at TEXT,
    last_used_at TEXT,
    rate_limit_per_minute INTEGER DEFAULT 60
);
```

### Indexes

- `idx_api_keys_hash` - Fast key lookup
- `idx_api_keys_active` - Filter active keys

## Next Steps

1. ✅ Database is ready (automatic on first run)
2. ✅ All migrations applied (automatic)
3. ⏭️ Create your first API key (see above)
4. ⏭️ Start using the API with your key

## Summary

**No manual database setup needed!** Just run:

```bash
cargo run --release
```

Everything else happens automatically. The only manual step is creating your first API key after the application starts.

