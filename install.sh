#!/bin/bash

set -e

INSTALL_DIR="/usr/local/bin"
BINARY_NAME="oyta"
TEMP_DIR=$(mktemp -d)

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

detect_arch() {
    local arch
    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        *)
            echo "unsupported"
            ;;
    esac
}

detect_libc() {
    if ldd --version 2>&1 | grep -qi musl; then
        echo "musl"
    else
        echo "gnu"
    fi
}

get_download_url() {
    local arch="$1"
    local libc="$2"
    
    case "$arch" in
        x86_64)
            echo "https://github.com/user-attachments/files/27156160/x86_64-unknown-linux-musl.zip"
            ;;
        aarch64)
            if [ "$libc" = "musl" ]; then
                echo "https://github.com/user-attachments/files/27156152/aarch64-unknown-linux-musl.zip"
            else
                echo "https://github.com/user-attachments/files/27156135/aarch64-unknown-linux-gnu.zip"
            fi
            ;;
        *)
            echo ""
            ;;
    esac
}

echo "========================================="
echo "       Oyta PHP Installer"
echo "========================================="

ARCH=$(detect_arch)
if [ "$ARCH" = "unsupported" ]; then
    echo "Error: Unsupported architecture: $(uname -m)"
    exit 1
fi

LIBC=$(detect_libc)
echo "Detected architecture: $ARCH"
echo "Detected libc: $LIBC"

DOWNLOAD_URL=$(get_download_url "$ARCH" "$LIBC")
if [ -z "$DOWNLOAD_URL" ]; then
    echo "Error: Could not determine download URL for your system"
    exit 1
fi

echo "Download URL: $DOWNLOAD_URL"
echo ""

ZIP_FILE="$TEMP_DIR/oyta.zip"
echo "Downloading..."
if command -v curl &> /dev/null; then
    curl -fsSL "$DOWNLOAD_URL" -o "$ZIP_FILE"
elif command -v wget &> /dev/null; then
    wget -q "$DOWNLOAD_URL" -O "$ZIP_FILE"
else
    echo "Error: Neither curl nor wget is available"
    exit 1
fi

echo "Extracting..."
unzip -q -o "$ZIP_FILE" -d "$TEMP_DIR"

BINARY_PATH="$TEMP_DIR/$BINARY_NAME"
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found after extraction"
    exit 1
fi

echo "Installing to $INSTALL_DIR..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
else
    echo "Requires sudo to install to $INSTALL_DIR"
    sudo mv "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
    sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
fi

echo ""
echo "========================================="
echo "Installation completed successfully!"
echo "Binary installed to: $INSTALL_DIR/$BINARY_NAME"
echo ""

if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo "Note: $INSTALL_DIR is not in your PATH"
    echo "Please add it to your PATH or restart your terminal"
fi

echo "Run 'oyta --help' to get started"
echo "========================================="
