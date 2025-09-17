#!/bin/bash

echo "📦 Preparing Servo-Stylo Project for Linux Transfer"
echo "=================================================="

# Create transfer directory
TRANSFER_DIR="stylo-compute-linux-ready"
mkdir -p "$TRANSFER_DIR"

echo "📁 Copying project files..."

# Copy all source files
cp -r src/ "$TRANSFER_DIR/"
cp -r examples/ "$TRANSFER_DIR/"

# Copy configuration and documentation
cp Cargo.toml "$TRANSFER_DIR/"
cp *.md "$TRANSFER_DIR/" 2>/dev/null || true
cp *.sh "$TRANSFER_DIR/" 2>/dev/null || true
cp *.toml "$TRANSFER_DIR/" 2>/dev/null || true

# Copy the real servo engine
cp enable_real_servo.rs "$TRANSFER_DIR/" 2>/dev/null || true

echo "🔧 Creating Linux-specific configuration..."

# Create Linux servo config template
cat > "$TRANSFER_DIR/servo_config_linux.toml" << 'EOF'
[servo]
# Update this path after building Servo on Linux
executable_path = "/home/user/servo/target/debug/servo"
mode = "real"

[integration]
enable_real_integration = true
use_javascript_injection = true

[debug]
verbose_logging = true
show_servo_output = true
EOF

# Create Linux setup script
cat > "$TRANSFER_DIR/setup_linux.sh" << 'EOF'
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
EOF

chmod +x "$TRANSFER_DIR/setup_linux.sh"

# Update Cargo.toml with missing dependencies
cat >> "$TRANSFER_DIR/Cargo.toml" << 'EOF'

# Additional dependencies for real Servo integration
[dependencies.tempfile]
version = "3.8"

[dependencies.uuid]
version = "1.0"
features = ["v4"]

[dependencies.which]
version = "4.4"
EOF

echo "📋 Creating transfer instructions..."

cat > "$TRANSFER_DIR/TRANSFER_INSTRUCTIONS.md" << 'EOF'
# 🚀 Transfer Instructions

## On Current Machine (macOS):
```bash
# Create tarball
tar -czf stylo-compute-linux.tar.gz stylo-compute-linux-ready/

# Transfer to Linux machine
scp stylo-compute-linux.tar.gz user@linux-machine:/home/user/
```

## On Linux Machine:
```bash
# Extract
cd /home/user/
tar -xzf stylo-compute-linux.tar.gz
cd stylo-compute-linux-ready/

# Run setup script
chmod +x setup_linux.sh
./setup_linux.sh

# Test real integration
cargo run --example servo_integration_demo
```

## Expected Output:
```
✅ Real Servo integration enabled with custom path: /home/user/servo/target/debug/servo
🔄 Querying real Servo process for computed styles...
   Using genuine Stylo APIs via Servo's getComputedStyle()
```

This means you're now using REAL Servo-Stylo integration!
EOF

# Create tarball
echo "📦 Creating transfer package..."
tar -czf stylo-compute-linux.tar.gz "$TRANSFER_DIR/"

echo "✅ Transfer package ready!"
echo ""
echo "📋 Next Steps:"
echo "1. Transfer stylo-compute-linux.tar.gz to your Linux machine"
echo "2. Extract: tar -xzf stylo-compute-linux.tar.gz"
echo "3. Run: cd stylo-compute-linux-ready && ./setup_linux.sh"
echo ""
echo "📁 Package contents:"
ls -la stylo-compute-linux.tar.gz
echo ""
echo "📖 See TRANSFER_INSTRUCTIONS.md in the package for detailed steps"
