#!/bin/bash

echo "🚀 Simple Servo-Stylo Setup (Servo Already Installed)"
echo "===================================================="

# Get Servo path from user
if [ -z "$1" ]; then
    echo "Usage: $0 /path/to/servo/executable"
    echo ""
    echo "Example:"
    echo "  $0 /usr/local/bin/servo"
    echo "  $0 /home/user/servo/target/debug/servo"
    echo "  $0 /opt/servo/servo"
    echo ""
    echo "Or if servo is in PATH, just use:"
    echo "  $0 \$(which servo)"
    exit 1
fi

SERVO_PATH="$1"

# Verify Servo exists and is executable
if [ ! -f "$SERVO_PATH" ]; then
    echo "❌ Servo executable not found at: $SERVO_PATH"
    exit 1
fi

if [ ! -x "$SERVO_PATH" ]; then
    echo "❌ Servo executable is not executable: $SERVO_PATH"
    echo "   Try: chmod +x $SERVO_PATH"
    exit 1
fi

echo "✅ Found Servo executable: $SERVO_PATH"

# Test Servo
echo "🧪 Testing Servo executable..."
if "$SERVO_PATH" --version >/dev/null 2>&1; then
    echo "✅ Servo is working!"
    "$SERVO_PATH" --version
else
    echo "⚠️  Servo executable found but may have issues"
    echo "   Continuing anyway..."
fi

echo ""

# Install Rust if needed (lightweight check)
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source ~/.cargo/env
    echo "✅ Rust installed"
else
    echo "✅ Rust already installed"
fi

echo ""

# Create configuration file
echo "⚙️  Creating Servo configuration..."
cat > servo_config.toml << EOF
[servo]
executable_path = "$SERVO_PATH"
mode = "real"

[integration]
enable_real_integration = true
use_javascript_injection = true

[debug]
verbose_logging = true
show_servo_output = true
EOF

echo "✅ Configuration created: servo_config.toml"
echo "   Servo path: $SERVO_PATH"

echo ""

# Test the integration
echo "🧪 Testing real Servo integration..."
echo "   Building project..."

if cargo build >/dev/null 2>&1; then
    echo "✅ Project built successfully"
    
    echo "   Running integration test..."
    if cargo run --example servo_integration_demo 2>/dev/null | grep -q "Real Servo integration enabled"; then
        echo "✅ Real Servo integration is working!"
        echo ""
        echo "🎉 Setup Complete!"
        echo ""
        echo "📋 What's enabled:"
        echo "   ✅ Real Servo executable: $SERVO_PATH"
        echo "   ✅ Real Stylo API integration"
        echo "   ✅ Genuine CSS computation"
        echo ""
        echo "🚀 To use real integration in your code:"
        echo "   use stylo_compute::ServoStyleEngineReal;"
        echo "   let mut engine = ServoStyleEngineReal::new()?;"
        echo ""
        echo "🧪 Test commands:"
        echo "   cargo run --example servo_integration_demo"
        echo "   cargo run --example real_servo_demo"
    else
        echo "⚠️  Integration test didn't show expected output"
        echo "   But configuration is created. Try running manually:"
        echo "   cargo run --example servo_integration_demo"
    fi
else
    echo "❌ Project build failed"
    echo "   You may need to install additional dependencies"
    echo "   But configuration file is created at: servo_config.toml"
fi

echo ""
echo "📁 Configuration file contents:"
cat servo_config.toml
