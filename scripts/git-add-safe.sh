#!/bin/bash
# Safe git add script that only adds files from the current jamey-3.0 directory
# This prevents accidentally adding files from the parent git repository

set -e

# Get the repository root
REPO_ROOT=$(git rev-parse --show-toplevel)

# Verify we're in the jamey-3.0 repository
if [[ "$REPO_ROOT" != *"jamey-3.0"* ]]; then
    echo "Error: This script must be run from within the jamey-3.0 repository"
    exit 1
fi

# Use git add . (without -A) to only add files from current directory
# This respects .gitignore and doesn't pick up files from parent repos
echo "Adding files from jamey-3.0 directory..."
git add .

# Show what was staged
echo ""
echo "Staged files:"
git status --short

