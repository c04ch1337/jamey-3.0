# Database Setup for Jamey 3.0

## Issue: "unable to open database file" Error

If you're getting this error, you likely need to install the SQLite development libraries.

## Solution

### Install SQLite Development Libraries

Run this command (requires sudo):

```bash
sudo apt-get update
sudo apt-get install -y libsqlite3-dev
```

### Verify Installation

After installing, try running the application again:

```bash
cargo run
```

## What's Happening

- **SQLite is embedded**: SQLx uses the system SQLite library, not a bundled version
- **Development headers needed**: Rust needs the `libsqlite3-dev` package to compile SQLx with SQLite support
- **No separate database server**: SQLite is file-based, so no database server installation is needed
- **Database file location**: The database will be created at `data/jamey.db` automatically

## Environment Variables

**You don't need any environment variables** for the basic setup. The database path is hardcoded to `data/jamey.db` relative to where you run the application.

If you want to customize the database location, you can modify `src/db/mod.rs` to read from an environment variable:

```rust
let db_path = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| "sqlite://data/jamey.db".to_string());
```

## Migration Files

The application includes a migration file at `migrations/20241114000000_init.sql` that will be automatically applied when you first run the application.

## Troubleshooting

1. **Permission errors**: Make sure the `data/` directory is writable:
   ```bash
   chmod 755 data/
   ```

2. **Path issues**: The database path is resolved to an absolute path automatically, so running from different directories should work.

3. **Migration errors**: If migrations fail, you can manually check the database:
   ```bash
   sqlite3 data/jamey.db ".tables"
   ```

