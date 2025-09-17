#!/bin/bash

echo "🐧 Setting up Servo-Stylo on Linux"
echo "================================="

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source ~/.cargo/env
fi

# Install system dependencies (Ubuntu/Debian)
if command -v apt &> /dev/null; then
    echo "📦 Installing system dependencies (Ubuntu/Debian)..."
    sudo apt update
    sudo apt install -y \
        git curl build-essential cmake pkg-config \
        libssl-dev libfreetype6-dev libfontconfig1-dev \
        libglib2.0-dev libcairo2-dev libpango1.0-dev \
        libatk1.0-dev libgdk-pixbuf2.0-dev libgtk-3-dev \
        libharfbuzz-dev python3 python3-pip libasound2-dev \
        libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
        libxkbcommon-dev libwayland-dev libdbus-1-dev libudev-dev
fi

# Install system dependencies (CentOS/RHEL/Fedora)
if command -v dnf &> /dev/null; then
    echo "📦 Installing system dependencies (Fedora/CentOS)..."
    sudo dnf install -y \
        git curl gcc gcc-c++ cmake pkg-config \
        openssl-devel freetype-devel fontconfig-devel \
        glib2-devel cairo-devel pango-devel \
        atk-devel gdk-pixbuf2-devel gtk3-devel \
        harfbuzz-devel python3 python3-pip alsa-lib-devel \
        libxcb-devel libxkbcommon-devel wayland-devel dbus-devel
fi

echo "🏗️  Cloning and building Servo..."
cd ..
if [ ! -d "servo" ]; then
    git clone https://github.com/servo/servo.git
fi

cd servo/
echo "Building Servo (this may take 30-60 minutes)..."
./mach build --dev

if [ -f "target/debug/servo" ]; then
    echo "✅ Servo built successfully!"
    SERVO_PATH="$(pwd)/target/debug/servo"
    
    # Update configuration
    cd ../stylo-compute-linux-ready/
    sed -i "s|/home/user/servo/target/debug/servo|$SERVO_PATH|g" servo_config_linux.toml
    cp servo_config_linux.toml servo_config.toml
    
    echo "✅ Configuration updated with actual Servo path: $SERVO_PATH"
    
    # Test the integration
    echo "🧪 Testing real Servo integration..."
    cargo run --example servo_integration_demo
    
else
    echo "❌ Servo build failed. Check the output above."
    exit 1
fi

echo "🎉 Setup complete! Real Servo-Stylo integration is ready."
