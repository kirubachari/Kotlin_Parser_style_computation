# 🐧 Linux Setup Guide for Servo-Stylo Integration

This guide helps you set up and run the Servo-Stylo CSS computation project on a Linux machine.

## 📦 Project Transfer

### Step 1: Transfer Project Files
```bash
# On your current machine, create a tarball
tar -czf stylo-compute-project.tar.gz stylo-compute/

# Transfer to Linux machine (replace with your details)
scp stylo-compute-project.tar.gz user@linux-machine:/home/user/

# On Linux machine, extract
cd /home/user/
tar -xzf stylo-compute-project.tar.gz
cd stylo-compute/
```

## 🔧 Linux Dependencies Installation

### For Ubuntu/Debian:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install system dependencies for Servo
sudo apt update
sudo apt install -y \
    git curl \
    build-essential cmake pkg-config \
    libssl-dev libfreetype6-dev libfontconfig1-dev \
    libglib2.0-dev libcairo2-dev libpango1.0-dev \
    libatk1.0-dev libgdk-pixbuf2.0-dev libgtk-3-dev \
    libharfbuzz-dev \
    python3 python3-pip \
    libasound2-dev \
    libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libwayland-dev

# Install additional dependencies
sudo apt install -y \
    libdbus-1-dev \
    libudev-dev \
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev
```

### For CentOS/RHEL/Fedora:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install system dependencies
sudo dnf install -y \
    git curl \
    gcc gcc-c++ cmake pkg-config \
    openssl-devel freetype-devel fontconfig-devel \
    glib2-devel cairo-devel pango-devel \
    atk-devel gdk-pixbuf2-devel gtk3-devel \
    harfbuzz-devel \
    python3 python3-pip \
    alsa-lib-devel \
    libxcb-devel libxkbcommon-devel wayland-devel

# Additional dependencies
sudo dnf install -y \
    dbus-devel \
    systemd-devel \
    gstreamer1-devel gstreamer1-plugins-base-devel
```

## 🏗️ Building Servo on Linux

### Step 1: Clone Servo
```bash
# Clone Servo repository
cd /home/user/
git clone https://github.com/servo/servo.git
cd servo/

# Install Python dependencies
pip3 install --user mach
```

### Step 2: Build Servo
```bash
# Build Servo (this takes 30-60 minutes)
./mach build --dev

# Verify build
ls -la target/debug/servo
./target/debug/servo --version
```

### Step 3: Add Servo to PATH (Optional)
```bash
# Add to ~/.bashrc or ~/.zshrc
echo 'export PATH="/home/user/servo/target/debug:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Test
which servo
servo --version
```

## 🚀 Enable Real Servo Integration

### Step 1: Update Configuration
```bash
cd /home/user/stylo-compute/

# Create configuration file
cat > servo_config.toml << EOF
[servo]
executable_path = "/home/user/servo/target/debug/servo"
mode = "real"

[integration]
enable_real_integration = true
use_javascript_injection = true

[debug]
verbose_logging = true
show_servo_output = true
EOF
```

### Step 2: Update Project Dependencies
```bash
# Add missing dependencies to Cargo.toml
cat >> Cargo.toml << EOF

# Additional dependencies for real Servo integration
[dependencies.tempfile]
version = "3.8"

[dependencies.uuid]
version = "1.0"
features = ["v4"]

[dependencies.which]
version = "4.4"
EOF
```

## 🧪 Testing Real Integration

### Step 1: Test Current Implementation
```bash
# Test simulation mode first
cargo run --example servo_integration_demo

# Test main application
cargo run
```

### Step 2: Test Real Servo Integration
```bash
# This should now use real Servo
cargo run --example servo_integration_demo

# You should see:
# ✅ Real Servo integration enabled with custom path: /home/user/servo/target/debug/servo
# 🔄 Querying real Servo process for computed styles...
# Using genuine Stylo APIs via Servo's getComputedStyle()
```

## 🔍 Troubleshooting

### Common Issues:

1. **Servo build fails**:
   ```bash
   # Check system dependencies
   ./mach bootstrap
   
   # Clean and rebuild
   ./mach clean
   ./mach build --dev
   ```

2. **Missing dependencies**:
   ```bash
   # Install additional packages
   sudo apt install -y libssl-dev pkg-config
   ```

3. **Permission issues**:
   ```bash
   # Fix permissions
   chmod +x /home/user/servo/target/debug/servo
   ```

4. **Path issues**:
   ```bash
   # Use absolute path in servo_config.toml
   executable_path = "/home/user/servo/target/debug/servo"
   ```

## 📋 Verification Checklist

- [ ] Rust installed and working
- [ ] System dependencies installed
- [ ] Servo cloned and built successfully
- [ ] Servo executable exists and runs
- [ ] Project transferred and dependencies updated
- [ ] Configuration file created
- [ ] Real integration test passes

## 🎯 Expected Output

When real integration works, you should see:
```
✅ Real Servo integration enabled with custom path: /home/user/servo/target/debug/servo
🔄 Querying real Servo process for computed styles...
   Using genuine Stylo APIs via Servo's getComputedStyle()
   Selector: .title
   Property: color
✅ Real computed color: rgb(255, 0, 0)
```

This indicates that your code is now using genuine Servo-Stylo integration instead of simulation!
