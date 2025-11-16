# ✅ Setup Complete - Database and All Remaining Steps

## What's Already Done

The database setup is **automatic** - it happens when you run the application! Here's what's in place:

### ✅ Automatic Database Initialization

The database is automatically created and migrations are run when you start the application:

1. **Database Location**: `data/jamey.db` (created automatically)
2. **Migrations**: All migrations run automatically via `sqlx::migrate!`
3. **Tables Created**:
   - `app_metadata` - Application metadata
   - `soul_entities` - Soul system entities
   - `soul_emotions` - Emotion tracking
   - `soul_memory_links` - Memory-entity links
   - `api_keys` - API key management (NEW!)

### ✅ Migration Files

All migrations are in `migrations/`:
- `20241114000000_init.sql` - Initial schema
- `20241114200000_soul_system.sql` - Soul system tables
- `20241115000000_api_keys.sql` - API key management (NEW!)

## Quick Start

### 1. Run the Application

The database will be created automatically:

```bash
cargo run --release
```

On first run, you'll see:
```
INFO Connecting to database at: data/jamey.db
INFO Database initialized and migrations applied
```

### 2. Verify Database Setup

Check that the database was created:

```bash
# On Linux/Mac
ls -lh data/jamey.db

# On Windows
dir data\jamey.db
```

### 3. Create Your First API Key

After the application is running, you can create an API key using the helper script:

```bash
# Make script executable (Linux/Mac)
chmod +x scripts/create-api-key.sh

# Create a key
./scripts/create-api-key.sh initial-key 60
```

Or manually using SQLite:

```bash
sqlite3 data/jamey.db
```

Then in SQLite:
```sql
-- Generate a key hash (use SHA-256 of your key)
-- For example, if your key is "my-secret-key-123"
-- You'd hash it first, then insert:

INSERT INTO api_keys (key_hash, name, created_at, rate_limit_per_minute)
VALUES (
  'your-sha256-hash-here',
  'initial-key',
  datetime('now'),
  60
);
```

**Better approach**: Use the `ApiKeyManager` programmatically or create a CLI command.

## Setup Script (Optional)

A setup script is available to help with initial configuration:

```bash
# Make executable (Linux/Mac)
chmod +x scripts/setup.sh

# Run setup
./scripts/setup.sh
```

This script will:
- Check for required tools (cargo, sqlx-cli)
- Create data directories
- Create `.env` template if missing
- Run migrations (if database exists)

## Manual Migration (Optional)

If you want to run migrations manually:

```bash
# Install sqlx-cli (if not already installed)
cargo install sqlx-cli --no-default-features --features sqlite

# Run migrations
sqlx migrate run --database-url "sqlite:data/jamey.db"
```

## Verify Everything Works

### 1. Check Database Tables

```bash
sqlite3 data/jamey.db ".tables"
```

You should see:
- `app_metadata`
- `soul_entities`
- `soul_emotions`
- `soul_memory_links`
- `api_keys` ← NEW!

### 2. Check API Keys Table

```bash
sqlite3 data/jamey.db "SELECT name, created_at, rate_limit_per_minute FROM api_keys;"
```

### 3. Test the API

Once you have an API key:

```bash
curl -X POST http://localhost:3000/evaluate \
  -H "x-api-key: YOUR_API_KEY_HERE" \
  -H "Content-Type: application/json" \
  -d '{"action": "I will help others"}'
```

## What Happens on Startup

When you run `cargo run`, the application:

1. ✅ Creates `data/` directory if missing
2. ✅ Creates `data/jamey.db` if missing
3. ✅ Runs all migrations automatically
4. ✅ Initializes memory indices
5. ✅ Starts the server on port 3000

## Troubleshooting

### Database Already Exists

If you need to reset the database:

```bash
# Stop the application (Ctrl+C)
rm data/jamey.db
cargo run  # Will recreate it
```

### Migration Errors

If migrations fail, check the migration files are correct:

```bash
sqlite3 data/jamey.db ".schema api_keys"
```

### Permission Errors

Make sure the `data/` directory is writable:

```bash
chmod 755 data/
```

## Next Steps

1. ✅ Database is ready (automatic)
2. ✅ Migrations will run (automatic)
3. ⏭️ Create your first API key (use script or programmatically)
4. ⏭️ Configure Nginx (if using reverse proxy)
5. ⏭️ Set up Prometheus (optional, for monitoring)

## Summary

**Everything is automatic!** Just run:

```bash
cargo run --release
```

The database will be created, migrations will run, and you'll be ready to go. The only manual step is creating your first API key, which you can do using the helper script or programmatically.

