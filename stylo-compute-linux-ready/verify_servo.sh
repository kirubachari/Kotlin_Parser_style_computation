#!/bin/bash

echo "🔍 Servo Installation Verification Script"
echo "========================================"

# 1. Check if Servo is in PATH
echo "1. Checking if Servo is in PATH..."
if command -v servo &> /dev/null; then
    echo "✅ Servo found in PATH: $(which servo)"
    echo "   Version: $(servo --version 2>/dev/null || echo 'Version check failed')"
else
    echo "❌ Servo not found in PATH"
fi

echo ""

# 2. Check for Servo in current project
echo "2. Checking for Servo in current project..."
if [ -f "servo/target/release/servo" ]; then
    echo "✅ Servo executable found: servo/target/release/servo"
    echo "   Size: $(ls -lh servo/target/release/servo | awk '{print $5}')"
    echo "   Testing: $(servo/target/release/servo --version 2>/dev/null || echo 'Version check failed')"
elif [ -f "servo/target/debug/servo" ]; then
    echo "✅ Servo debug executable found: servo/target/debug/servo"
    echo "   Size: $(ls -lh servo/target/debug/servo | awk '{print $5}')"
    echo "   Testing: $(servo/target/debug/servo --version 2>/dev/null || echo 'Version check failed')"
else
    echo "❌ No Servo executable found in project"
fi

echo ""

# 3. Check for Servo source directory
echo "3. Checking Servo source directory..."
if [ -d "servo" ]; then
    echo "✅ Servo source directory found"
    echo "   Has mach build script: $([ -f servo/mach ] && echo 'Yes' || echo 'No')"
    echo "   Has Cargo.toml: $([ -f servo/Cargo.toml ] && echo 'Yes' || echo 'No')"
    echo "   Build status:"
    if [ -d "servo/target/release" ]; then
        echo "     - Release build directory exists"
        echo "     - Build artifacts: $(ls servo/target/release/ | wc -l) files"
    fi
    if [ -d "servo/target/debug" ]; then
        echo "     - Debug build directory exists"
        echo "     - Build artifacts: $(ls servo/target/debug/ | wc -l) files"
    fi
else
    echo "❌ No Servo source directory found"
fi

echo ""

# 4. Search for Servo executables system-wide (limited search)
echo "4. Searching for Servo executables..."
echo "   Searching in common locations..."

# Check common installation paths
SEARCH_PATHS=(
    "/usr/local/bin"
    "/opt/homebrew/bin"
    "$HOME/.cargo/bin"
    "$HOME/bin"
    "$HOME/.local/bin"
)

for path in "${SEARCH_PATHS[@]}"; do
    if [ -f "$path/servo" ]; then
        echo "✅ Found Servo at: $path/servo"
        echo "   Version: $($path/servo --version 2>/dev/null || echo 'Version check failed')"
    fi
done

echo ""

# 5. Check Rust toolchain (required for building Servo)
echo "5. Checking Rust toolchain..."
if command -v rustc &> /dev/null; then
    echo "✅ Rust compiler: $(rustc --version)"
    echo "✅ Cargo: $(cargo --version)"
else
    echo "❌ Rust toolchain not found (required for building Servo)"
fi

echo ""

# 6. Recommendations
echo "6. Recommendations:"
echo "=================="

if command -v servo &> /dev/null || [ -f "servo/target/release/servo" ] || [ -f "servo/target/debug/servo" ]; then
    echo "✅ Servo is available! You can proceed with real integration."
else
    echo "❌ Servo needs to be built. Run one of these commands:"
    if [ -d "servo" ]; then
        echo "   cd servo && ./mach build --release    # For optimized build (recommended)"
        echo "   cd servo && ./mach build --dev        # For debug build (faster compilation)"
    else
        echo "   git clone https://github.com/servo/servo.git"
        echo "   cd servo && ./mach build --release"
    fi
fi

echo ""
echo "🎯 Next Steps:"
echo "1. If Servo is found, update ServoStyleEngine to use the correct path"
echo "2. If Servo needs building, run the build command above"
echo "3. Test the integration with: cargo run --example servo_integration_demo"
