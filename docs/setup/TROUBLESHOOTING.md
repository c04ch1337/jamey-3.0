# Troubleshooting Guide

## Build Errors

### OpenSSL / pkg-config Errors

**Error**: `Could not find openssl via pkg-config` or `pkg-config command could not be found`

**Solution**: The project now uses `rustls` instead of OpenSSL, so you don't need to install OpenSSL development libraries. If you still see this error:

1. **Option 1 (Recommended)**: The project is already configured to use `rustls` - just rebuild:
   ```bash
   cargo clean
   cargo build
   ```

2. **Option 2**: If you prefer OpenSSL (not recommended), install:
   ```bash
   sudo apt-get install pkg-config libssl-dev
   ```

### SQLite Database Errors

**Error**: `unable to open database file`

**Solution**: Install SQLite development libraries:
```bash
sudo apt-get install libsqlite3-dev
```

### Missing Dependencies

If you see errors about missing system libraries, install:

```bash
# For SQLite
sudo apt-get install libsqlite3-dev

# For pkg-config (if needed)
sudo apt-get install pkg-config

# For OpenSSL (only if not using rustls)
sudo apt-get install libssl-dev
```

## Runtime Errors

### OpenRouter API Key Missing

**Error**: `OPENROUTER_API_KEY environment variable is required`

**Solution**: 
1. Create a `.env` file in the project root
2. Add: `OPENROUTER_API_KEY=your-key-here`
3. Get your key from: https://openrouter.ai/keys

See `API_KEY_SETUP.md` for detailed instructions.

### Database Connection Errors

**Error**: `unable to open database file`

**Solution**:
1. Ensure the `data/` directory exists and is writable:
   ```bash
   mkdir -p data
   chmod 755 data
   ```
2. Check file permissions
3. Ensure SQLite is installed: `sudo apt-get install libsqlite3-dev`

## Common Issues

### Port Already in Use

**Error**: `Address already in use`

**Solution**: 
- Change the port in `src/main.rs` (default is 3000)
- Or kill the process using the port:
  ```bash
  lsof -ti:3000 | xargs kill -9
  ```

### Cargo Build Fails

**Error**: Various compilation errors

**Solution**:
1. Update Rust: `rustup update stable`
2. Clean and rebuild: `cargo clean && cargo build`
3. Check Rust version: `rustc --version` (should be 1.70+)

### Frontend Won't Start

**Error**: Frontend build or runtime errors

**Solution**:
1. Install dependencies: `cd frontend && npm install`
2. Check Node.js version: `node --version` (should be 18+)
3. Clear cache: `rm -rf node_modules package-lock.json && npm install`

## Getting Help

If you encounter other issues:

1. Check the logs: Set `RUST_LOG=debug` for verbose output
2. Check the documentation:
   - `README.md` - General setup
   - `API_KEY_SETUP.md` - OpenRouter configuration
   - `SETUP_DATABASE.md` - Database setup
3. Verify your environment:
   ```bash
   rustc --version
   cargo --version
   node --version
   npm --version
   ```

