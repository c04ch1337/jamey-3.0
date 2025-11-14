# Fix Git Cache and Embedded Repository Issues

## Problem

When running `git add -A`, thousands of cache files and embedded git repositories are being added, particularly from:
- `.config/Cursor/User/globalStorage/kilocode.kilo-code/tasks/*/checkpoints` (embedded git repos)
- Rust build cache files
- Various IDE cache files

## Solution

### 1. Updated .gitignore

The `.gitignore` file has been updated to exclude:

```gitignore
# Cursor IDE configuration and cache
.config/
.config/**
.config/Cursor/
.config/Cursor/**

# Rust incremental compilation cache
**/incremental/
**/*.rlib
**/*.rmeta
**/*.pdb

# Cache files
*.cache
**/*.cache
.cache/
**/.cache/

# Vite cache
frontend/.vite/
frontend/node_modules/.cache/

# TypeScript cache
*.tsbuildinfo
.tsbuildinfo
```

### 2. Remove Already-Staged Files

If you've already staged cache files, remove them:

```bash
# Remove .config if it was added
git rm -r --cached .config/ 2>/dev/null || true

# Remove target/ if it was added
git rm -r --cached target/ 2>/dev/null || true

# Remove any cache files
git rm --cached **/*.cache 2>/dev/null || true
```

### 3. Fix Corrupted Git Index

If you see "confused by unstable object source data" error:

```bash
# Reset to clean state
git reset --hard HEAD

# Clean untracked files (be careful!)
git clean -fd

# Rebuild git index
rm -f .git/index
git reset
```

### 4. Safe Git Add

Instead of `git add -A`, use more specific commands:

```bash
# Add only tracked changes
git add -u

# Add specific files/directories
git add src/ frontend/ docs/ *.md *.toml

# Or add everything except ignored files
git add .
```

### 5. Verify Before Committing

Always check what you're about to commit:

```bash
# See what's staged
git status --short

# See detailed changes
git diff --cached

# Count files to be committed
git diff --cached --name-only | wc -l
```

## Prevention

### Always Run from Project Root

Make sure you're in the project root (`/home/vendetta/jamey-3.0`) when running git commands:

```bash
cd /home/vendetta/jamey-3.0
git status
```

### Use Specific Paths

Instead of `git add -A`, be explicit:

```bash
# Good
git add src/ frontend/ docs/ *.md

# Avoid
git add -A
```

### Check .gitignore Regularly

Verify your `.gitignore` is working:

```bash
# Test if a file would be ignored
git check-ignore -v path/to/file

# See what files are being tracked
git ls-files | grep -E "(cache|target|node_modules)"
```

## Current Status

✅ `.gitignore` updated to exclude:
- `.config/` directory (Cursor IDE cache)
- `target/` directory (Rust build artifacts)
- All cache files
- Incremental compilation files

✅ Git index should be clean now

## CRITICAL: Why `git add -A -- .` Fails

**The issue**: When you run `git add -A -- .` from within `jamey-3.0`, Git can incorrectly try to add files from outside the repository, including:
- `.config/Cursor/...` (Cursor IDE cache that may exist in parent directories)
- `.nvm/` (Node Version Manager)
- `Agents/...` (other projects)

**Why this happens**: The `-A` flag with `-- .` can cause Git to misinterpret paths, especially when there are embedded git repositories or when paths are resolved unexpectedly.

### Safe Git Commands

**❌ NEVER USE**: `git add -A -- .`  
**❌ NEVER USE**: `git add -A` (without specifying directory)

**✅ ALWAYS USE ONE OF THESE**:

**Option 1: Use the safe script (Recommended)**
```bash
./scripts/git-add-safe.sh
```

**Option 2: Use `git add .` (without `-A`)**
```bash
git add .
```

**Option 3: Add specific files**
```bash
git add path/to/file1 path/to/file2
```

## Next Steps

1. **Review staged files**:
   ```bash
   git status --short
   ```

2. **If clean, commit**:
   ```bash
   git commit -m "Update .gitignore to exclude cache files and IDE configs"
   ```

3. **Push**:
   ```bash
   git push origin main
   ```

