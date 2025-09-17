#!/bin/bash

echo "🔧 Servo Dependencies Setup Script"
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This script is designed for macOS. For other systems, please install dependencies manually."
    exit 1
fi

print_status "Installing Servo build dependencies for macOS..."

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    print_error "Homebrew is required but not installed."
    echo "Please install Homebrew first: https://brew.sh/"
    exit 1
fi

print_success "Homebrew found"

# Install required dependencies
print_status "Installing system dependencies..."

# Core dependencies for Servo
DEPENDENCIES=(
    "cmake"
    "pkg-config" 
    "freetype"
    "fontconfig"
    "glib"
    "cairo"
    "pango"
    "harfbuzz"
    "icu4c"
    "libpng"
    "jpeg"
    "webp"
    "openssl"
)

for dep in "${DEPENDENCIES[@]}"; do
    print_status "Installing $dep..."
    if brew list "$dep" &>/dev/null; then
        print_success "$dep already installed"
    else
        brew install "$dep"
        if [ $? -eq 0 ]; then
            print_success "$dep installed successfully"
        else
            print_warning "Failed to install $dep (may not be critical)"
        fi
    fi
done

# Set up environment variables for pkg-config
print_status "Setting up environment variables..."

# Get Homebrew prefix (different on Intel vs Apple Silicon)
HOMEBREW_PREFIX=$(brew --prefix)
print_status "Homebrew prefix: $HOMEBREW_PREFIX"

# Set PKG_CONFIG_PATH
export PKG_CONFIG_PATH="$HOMEBREW_PREFIX/lib/pkgconfig:$HOMEBREW_PREFIX/share/pkgconfig:$PKG_CONFIG_PATH"

# Create environment setup script
cat > servo_env.sh << EOF
#!/bin/bash
# Servo Build Environment Setup

# Homebrew paths
export HOMEBREW_PREFIX="$HOMEBREW_PREFIX"

# PKG_CONFIG paths
export PKG_CONFIG_PATH="$HOMEBREW_PREFIX/lib/pkgconfig:$HOMEBREW_PREFIX/share/pkgconfig:\$PKG_CONFIG_PATH"

# Library paths
export LIBRARY_PATH="$HOMEBREW_PREFIX/lib:\$LIBRARY_PATH"
export CPATH="$HOMEBREW_PREFIX/include:\$CPATH"

# ICU paths (often needed)
export ICU_ROOT="$HOMEBREW_PREFIX/opt/icu4c"
export PKG_CONFIG_PATH="$HOMEBREW_PREFIX/opt/icu4c/lib/pkgconfig:\$PKG_CONFIG_PATH"

# OpenSSL paths
export OPENSSL_ROOT_DIR="$HOMEBREW_PREFIX/opt/openssl"
export PKG_CONFIG_PATH="$HOMEBREW_PREFIX/opt/openssl/lib/pkgconfig:\$PKG_CONFIG_PATH"

# Freetype paths
export PKG_CONFIG_PATH="$HOMEBREW_PREFIX/opt/freetype/lib/pkgconfig:\$PKG_CONFIG_PATH"

echo "✅ Servo build environment configured"
echo "   PKG_CONFIG_PATH: \$PKG_CONFIG_PATH"
echo "   LIBRARY_PATH: \$LIBRARY_PATH"
EOF

chmod +x servo_env.sh
print_success "Environment setup script created: servo_env.sh"

# Test pkg-config
print_status "Testing pkg-config setup..."
source ./servo_env.sh

if pkg-config --exists glib-2.0; then
    print_success "glib-2.0 found by pkg-config"
    GLIB_VERSION=$(pkg-config --modversion glib-2.0)
    print_status "glib version: $GLIB_VERSION"
else
    print_error "glib-2.0 still not found by pkg-config"
    print_status "Trying to locate glib manually..."
    
    # Try to find glib in common locations
    GLIB_LOCATIONS=(
        "$HOMEBREW_PREFIX/lib/pkgconfig/glib-2.0.pc"
        "/usr/local/lib/pkgconfig/glib-2.0.pc"
        "/opt/homebrew/lib/pkgconfig/glib-2.0.pc"
    )
    
    for location in "${GLIB_LOCATIONS[@]}"; do
        if [ -f "$location" ]; then
            print_success "Found glib at: $location"
            break
        fi
    done
fi

echo ""
print_status "Attempting to build Servo with proper environment..."

# Try building at the external Servo location
EXTERNAL_SERVO="/Users/kiruba-2957/Development/Kotlin-Parser-Rust/servo"

if [ -d "$EXTERNAL_SERVO" ]; then
    print_status "Found external Servo installation: $EXTERNAL_SERVO"
    
    cd "$EXTERNAL_SERVO"
    
    if [ -f "./mach" ]; then
        print_status "Starting Servo build with proper environment..."
        
        # Source the environment and build
        source "$PWD/../stylo-compute/servo_env.sh"
        
        print_status "Building Servo (this may take 15-30 minutes)..."
        ./mach build --dev
        
        if [ $? -eq 0 ]; then
            print_success "Servo build completed successfully!"
            
            if [ -f "./target/debug/servo" ]; then
                SERVO_EXECUTABLE="$EXTERNAL_SERVO/target/debug/servo"
                print_success "Servo executable created: $SERVO_EXECUTABLE"
                
                # Create config pointing to this Servo
                cd "$PWD/../stylo-compute"
                cat > servo_config.toml << EOF
# Servo Configuration for Real Integration
[servo]
executable_path = "$SERVO_EXECUTABLE"
source_directory = "$EXTERNAL_SERVO"
mode = "real"

[integration]
enable_real_integration = true
use_javascript_injection = true

[paths]
servo_executable = "$SERVO_EXECUTABLE"
servo_source = "$EXTERNAL_SERVO"
EOF
                
                print_success "Configuration updated with working Servo path"
                
            else
                print_error "Build completed but executable not found"
            fi
        else
            print_error "Servo build failed even with dependencies installed"
            print_status "This may be due to Servo version compatibility issues"
        fi
    else
        print_error "No mach build script found in $EXTERNAL_SERVO"
    fi
else
    print_warning "External Servo directory not found: $EXTERNAL_SERVO"
fi

echo ""
print_success "Dependency setup completed!"
echo ""
echo "🎯 Next Steps:"
echo "=============="
echo "1. Source the environment: source servo_env.sh"
echo "2. Try building Servo manually if the automatic build failed"
echo "3. Use the ServoStyleEngine with the configured path"
echo ""
echo "📁 Files created:"
echo "- servo_env.sh (environment setup)"
echo "- servo_config.toml (configuration)"
echo ""
echo "🔧 Manual build command (if needed):"
echo "   cd $EXTERNAL_SERVO"
echo "   source ../stylo-compute/servo_env.sh"
echo "   ./mach build --dev"
