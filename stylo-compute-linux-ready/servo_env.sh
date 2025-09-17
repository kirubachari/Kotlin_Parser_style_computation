#!/bin/bash
# Servo Build Environment Setup

# Homebrew paths
export HOMEBREW_PREFIX="/opt/homebrew"

# PKG_CONFIG paths
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:/opt/homebrew/share/pkgconfig:$PKG_CONFIG_PATH"

# Library paths
export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH"
export CPATH="/opt/homebrew/include:$CPATH"

# ICU paths (often needed)
export ICU_ROOT="/opt/homebrew/opt/icu4c"
export PKG_CONFIG_PATH="/opt/homebrew/opt/icu4c/lib/pkgconfig:$PKG_CONFIG_PATH"

# OpenSSL paths
export OPENSSL_ROOT_DIR="/opt/homebrew/opt/openssl"
export PKG_CONFIG_PATH="/opt/homebrew/opt/openssl/lib/pkgconfig:$PKG_CONFIG_PATH"

# Freetype paths
export PKG_CONFIG_PATH="/opt/homebrew/opt/freetype/lib/pkgconfig:$PKG_CONFIG_PATH"

echo "✅ Servo build environment configured"
echo "   PKG_CONFIG_PATH: $PKG_CONFIG_PATH"
echo "   LIBRARY_PATH: $LIBRARY_PATH"
