# Git Best Practices for Jamey 3.0

## ⚠️ Important: Always Run Git Commands from Project Root

**Always** run git commands from `/home/vendetta/jamey-3.0`, not from parent directories.

```bash
# ✅ CORRECT - Run from project root
cd /home/vendetta/jamey-3.0
git add .
git commit -m "message"
git push

# ❌ WRONG - Don't run from parent directory
cd /home/vendetta
git add -A  # This will add files from parent directory!
```

## Safe Git Commands

### Instead of `git add -A`, use:

```bash
# Add only files in current directory and subdirectories
git add .

# Or be specific
git add src/ frontend/ docs/ *.md *.toml .gitignore

# Add only modified tracked files
git add -u
```

### Check Before Committing

```bash
# See what will be committed
git status --short

# Count files
git status --short | wc -l

# See file sizes
git status --short | xargs ls -lh 2>/dev/null | head -20
```

## What's Ignored

The `.gitignore` file excludes:

- ✅ `target/` - Rust build artifacts
- ✅ `.config/` - Cursor IDE configuration
- ✅ `data/` - Database and memory indices
- ✅ `node_modules/` - Node.js dependencies
- ✅ `*.cache` - All cache files
- ✅ `*.log` - Log files
- ✅ `.env` - Environment variables

## Verify .gitignore is Working

```bash
# Test if a path is ignored
git check-ignore -v path/to/file

# See all tracked files
git ls-files

# Count tracked files (should be ~70-100, not thousands)
git ls-files | wc -l
```

## If You Accidentally Added Cache Files

```bash
# Remove from staging (but keep files locally)
git reset HEAD path/to/cache/file

# Remove from git entirely (and delete locally - be careful!)
git rm --cached -r path/to/cache/directory
```

## Current Repository Status

- **Repository Root**: `/home/vendetta/jamey-3.0`
- **Remote**: `git@github.com:c04ch1337/jamey-3.0.git`
- **Branch**: `main`
- **Tracked Files**: ~71 files (should stay in this range)

## Quick Reference

```bash
# Always start here
cd /home/vendetta/jamey-3.0

# Check status
git status

# Add files safely
git add .

# Commit
git commit -m "Your message"

# Push
git push origin main
```

