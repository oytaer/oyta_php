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
            echo "https://github.com/user-attachments/files/27195349/oyta-linux-x64-musl.zip|oyta-linux-x64-musl"
            ;;
        aarch64)
            if [ "$libc" = "musl" ]; then
                echo "https://github.com/user-attachments/files/27195346/oyta-linux-arm64-musl.zip|oyta-linux-arm64-musl"
            else
                echo "https://github.com/user-attachments/files/27195339/oyta-linux-arm64.zip|oyta-linux-arm64"
            fi
            ;;
        *)
            echo ""
            ;;
    esac
}

install_unzip() {
    if command -v apt-get >/dev/null 2>&1; then
        echo "Installing unzip via apt-get..."
        sudo apt-get update -qq
        sudo apt-get install -y -qq unzip
    elif command -v yum >/dev/null 2>&1; then
        echo "Installing unzip via yum..."
        sudo yum install -y -q unzip
    elif command -v dnf >/dev/null 2>&1; then
        echo "Installing unzip via dnf..."
        sudo dnf install -y -q unzip
    elif command -v apk >/dev/null 2>&1; then
        echo "Installing unzip via apk..."
        sudo apk add --quiet unzip
    elif command -v pacman >/dev/null 2>&1; then
        echo "Installing unzip via pacman..."
        sudo pacman -S --noconfirm --quiet unzip
    else
        echo "Error: Could not detect package manager. Please install unzip manually."
        exit 1
    fi
}

ensure_unzip() {
    if ! command -v unzip >/dev/null 2>&1; then
        echo "unzip not found, installing..."
        install_unzip
    fi
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

DOWNLOAD_INFO=$(get_download_url "$ARCH" "$LIBC")
if [ -z "$DOWNLOAD_INFO" ]; then
    echo "Error: Could not determine download URL for your system"
    exit 1
fi

DOWNLOAD_URL=$(echo "$DOWNLOAD_INFO" | cut -d'|' -f1)
BINARY_IN_ZIP=$(echo "$DOWNLOAD_INFO" | cut -d'|' -f2)

echo "Download URL: $DOWNLOAD_URL"
echo "Binary name: $BINARY_IN_ZIP"
echo ""

ensure_unzip

ZIP_FILE="$TEMP_DIR/oyta.zip"
echo "Downloading..."
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o "$ZIP_FILE"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$DOWNLOAD_URL" -O "$ZIP_FILE"
else
    echo "Error: Neither curl nor wget is available"
    echo "Please install curl or wget first"
    exit 1
fi

echo "Extracting..."
unzip -q -o "$ZIP_FILE" -d "$TEMP_DIR"

BINARY_PATH="$TEMP_DIR/$BINARY_IN_ZIP"
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found after extraction (expected: $BINARY_IN_ZIP)"
    echo "Contents of temp dir:"
    ls -la "$TEMP_DIR"
    exit 1
fi

echo "Installing to $INSTALL_DIR..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
else
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
