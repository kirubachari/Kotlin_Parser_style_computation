#!/bin/bash

echo "⚡ Quick Servo Integration Setup"
echo "==============================="

# Get Servo path
if [ -z "$1" ]; then
    # Try to auto-detect
    if command -v servo >/dev/null 2>&1; then
        SERVO_PATH=$(which servo)
        echo "🔍 Auto-detected Servo in PATH: $SERVO_PATH"
    else
        echo "❌ Please provide the path to your Servo executable:"
        echo "   $0 /path/to/servo"
        echo ""
        echo "Examples:"
        echo "   $0 /usr/local/bin/servo"
        echo "   $0 /home/user/servo/target/debug/servo"
        echo "   $0 \$(which servo)"
        exit 1
    fi
else
    SERVO_PATH="$1"
fi

# Verify Servo exists
if [ ! -f "$SERVO_PATH" ]; then
    echo "❌ Servo not found at: $SERVO_PATH"
    exit 1
fi

echo "✅ Using Servo: $SERVO_PATH"

# Create config
cat > servo_config.toml << EOF
[servo]
executable_path = "$SERVO_PATH"
mode = "real"

[integration]
enable_real_integration = true
use_javascript_injection = true
EOF

echo "✅ Real Servo integration enabled!"
echo ""
echo "🚀 Now you can use:"
echo "   cargo run --example servo_integration_demo"
echo ""
echo "📋 In your code, use:"
echo "   ServoStyleEngineReal::new()  // Uses real Servo"
echo "   instead of:"
echo "   ServoStyleEngine::new()      // Uses simulation"
