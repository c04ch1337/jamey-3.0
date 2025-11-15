#!/bin/bash

# Fix MongoDB GPG key warning in Ubuntu
# This converts the old binary GPG key format to the new ASCII armored format

echo "Fixing MongoDB GPG key format..."

# Method 1: Try to export and re-import the key
if [ -f /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg ]; then
    echo "Found old MongoDB GPG key file"
    
    # Backup the old key
    sudo cp /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg.backup
    
    # Try to extract the key ID first
    KEY_ID=$(gpg --no-default-keyring --keyring /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg --list-keys 2>/dev/null | grep -oP '^\w+\s+\K[0-9A-F]{16}' | head -1)
    
    if [ -n "$KEY_ID" ]; then
        echo "Found key ID: $KEY_ID"
        # Export and convert to new format
        gpg --no-default-keyring --keyring /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg --export "$KEY_ID" 2>/dev/null | \
            sudo gpg --dearmor -o /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg.new
        
        if [ -f /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg.new ]; then
            sudo mv /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg.new /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg
            echo "✅ Key converted successfully"
        else
            echo "⚠️  Conversion failed, restoring backup"
            sudo mv /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg.backup /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg
        fi
    else
        echo "⚠️  Could not extract key ID. Trying alternative method..."
        # Alternative: Remove and re-add from MongoDB's key server
        echo "Removing old key and re-adding from MongoDB..."
        sudo rm /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg
        
        # Re-add MongoDB GPG key (you may need to adjust the key ID)
        curl -fsSL https://pgp.mongodb.com/server-6.0.asc | sudo gpg --dearmor -o /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg
        
        if [ -f /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg ]; then
            echo "✅ MongoDB key re-added successfully"
        else
            echo "❌ Failed to re-add key. Restoring backup..."
            sudo mv /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg.backup /etc/apt/trusted.gpg.d/mongodb-server-6.0.gpg
        fi
    fi
else
    echo "MongoDB GPG key file not found"
fi

echo ""
echo "Testing apt update..."
sudo apt update 2>&1 | grep -i mongodb || echo "✅ No MongoDB warnings found"

