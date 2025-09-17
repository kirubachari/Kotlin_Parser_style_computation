#!/bin/bash

echo "🔧 Building and Enabling Real Servo Integration"
echo "=============================================="

# Step 1: Install dependencies
echo "📦 Installing Servo build dependencies..."
if command -v brew >/dev/null 2>&1; then
    brew install cmake pkg-config glib cairo pango atk gdk-pixbuf gtk+3 harfbuzz
else
    echo "❌ Homebrew not found. Please install dependencies manually."
    exit 1
fi

# Step 2: Set up environment
echo "🌍 Setting up build environment..."
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH"
export LIBRARY_PATH="/opt/homebrew/lib:/usr/local/lib:$LIBRARY_PATH"
export CPATH="/opt/homebrew/include:/usr/local/include:$CPATH"

# Step 3: Build Servo
echo "🏗️  Building Servo (this may take 30-60 minutes)..."
cd /Users/kiruba-2957/Development/Kotlin-Parser-Rust/servo

if [ ! -f "./mach" ]; then
    echo "❌ Servo source not found at expected location"
    echo "   Please ensure Servo is cloned at: /Users/kiruba-2957/Development/Kotlin-Parser-Rust/servo"
    exit 1
fi

# Build Servo
./mach build --dev

# Step 4: Check if build succeeded
SERVO_EXECUTABLE="/Users/kiruba-2957/Development/Kotlin-Parser-Rust/servo/target/debug/servo"
if [ -f "$SERVO_EXECUTABLE" ]; then
    echo "✅ Servo built successfully!"
    echo "   Executable: $SERVO_EXECUTABLE"
    
    # Step 5: Create configuration for real integration
    cd /Users/kiruba-2957/Development/Kotlin-Parser-Rust/stylo-compute
    
    cat > servo_config.toml << EOF
[servo]
executable_path = "$SERVO_EXECUTABLE"
mode = "real"

[integration]
enable_real_integration = true
use_javascript_injection = true
EOF
    
    echo "✅ Configuration created: servo_config.toml"
    
    # Step 6: Test real integration
    echo "🧪 Testing real Servo integration..."
    cargo run --example servo_integration_demo
    
else
    echo "❌ Servo build failed"
    echo "   Check the build output above for errors"
    exit 1
fi

echo "🎉 Real Servo integration is now enabled!"
