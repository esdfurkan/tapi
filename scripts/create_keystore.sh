#!/bin/bash

# Android Keystore Auto-Generator
# This script creates a keystore file with random secure credentials

set -e

echo "=========================================="
echo "Android Keystore Auto-Generator"
echo "=========================================="

# Detect user home directory
USER_HOME="${HOME:-/root}"
KEYSTORE_DIR="$USER_HOME"
KEYSTORE_FILE="$KEYSTORE_DIR/upload-keystore.jks"
KEY_PROPERTIES_FILE="src-tauri/gen/android/key.properties"

# Check if running in CI
if [ "$CI" = "true" ]; then
    echo "Running in CI environment"
    # In CI, always create new keystore
    rm -f "$KEYSTORE_FILE"
else
    # Check if keystore already exists
    if [ -f "$KEYSTORE_FILE" ]; then
        echo "⚠️  Keystore already exists at: $KEYSTORE_FILE"
        read -p "Do you want to overwrite it? (yes/no): " OVERWRITE
        if [ "$OVERWRITE" != "yes" ]; then
            echo "Aborted. Using existing keystore."
            exit 0
        fi
        rm -f "$KEYSTORE_FILE"
    fi
fi

# Generate random secure passwords (64 characters)
generate_password() {
    openssl rand -base64 48 | tr -d "=+/" | cut -c1-64
}

STORE_PASSWORD=$(generate_password)
KEY_PASSWORD=$(generate_password)
KEY_ALIAS="upload"

# Get system information for certificate details
HOSTNAME=$(hostname 2>/dev/null || echo "android-builder")
USERNAME=$(whoami 2>/dev/null || echo "developer")

# Certificate details (can be customized)
CERT_CN="$USERNAME@$HOSTNAME"
CERT_OU="Development"
CERT_O="TAPI Translation Tool"
CERT_L="Unknown"
CERT_ST="Unknown"
CERT_C="US"

echo ""
echo "Generating keystore with the following details:"
echo "  Location: $KEYSTORE_FILE"
echo "  Alias: $KEY_ALIAS"
echo "  CN: $CERT_CN"
echo "  Organization: $CERT_O"
echo ""

# Generate keystore using keytool
keytool -genkeypair \
    -v \
    -keystore "$KEYSTORE_FILE" \
    -alias "$KEY_ALIAS" \
    -keyalg RSA \
    -keysize 2048 \
    -validity 10000 \
    -storepass "$STORE_PASSWORD" \
    -keypass "$KEY_PASSWORD" \
    -dname "CN=$CERT_CN, OU=$CERT_OU, O=$CERT_O, L=$CERT_L, ST=$CERT_ST, C=$CERT_C"

if [ ! -f "$KEYSTORE_FILE" ]; then
    echo "❌ Failed to create keystore!"
    exit 1
fi

echo "✅ Keystore created successfully!"
echo ""

# Create or update key.properties file
echo "Updating key.properties file..."

mkdir -p "$(dirname "$KEY_PROPERTIES_FILE")"

cat > "$KEY_PROPERTIES_FILE" << EOF
storePassword=$STORE_PASSWORD
keyPassword=$KEY_PASSWORD
keyAlias=$KEY_ALIAS
storeFile=$KEYSTORE_FILE
EOF

echo "✅ key.properties updated!"
echo ""

if [ "$CI" != "true" ]; then
    # Security reminder (skip in CI)
    echo "=========================================="
    echo "⚠️  SECURITY NOTICE"
    echo "=========================================="
    echo "Keystore location: $KEYSTORE_FILE"
    echo "Credentials saved to: $KEY_PROPERTIES_FILE"
    echo ""
    echo "⚠️  IMPORTANT:"
    echo "  1. Keep these files SECURE and PRIVATE"
    echo "  2. Add them to .gitignore (already done)"
    echo "  3. Backup the keystore file safely"
    echo "  4. You CANNOT recover the keystore if lost"
    echo "  5. All future app updates MUST use the same keystore"
    echo ""
fi

echo "✅ Setup complete! You can now build the Android app."
echo "=========================================="
