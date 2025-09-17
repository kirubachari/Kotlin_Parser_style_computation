# ⚡ Simple Servo Setup (Servo Already Installed)

Since you already have Servo installed, you only need to do 2 simple steps:

## 🎯 Quick Setup (30 seconds)

### Step 1: Create Configuration File
```bash
# Replace /path/to/your/servo with your actual Servo path
cat > servo_config.toml << EOF
[servo]
executable_path = "/path/to/your/servo"
mode = "real"

[integration]
enable_real_integration = true
use_javascript_injection = true
EOF
```

### Step 2: Test Real Integration
```bash
cargo run --example servo_integration_demo
```

**Expected output:**
```
✅ Real Servo integration enabled with custom path: /path/to/your/servo
🔄 Querying real Servo process for computed styles...
   Using genuine Stylo APIs via Servo's getComputedStyle()
```

## 🚀 Even Simpler: Use the Script

### Option 1: Auto-detect Servo in PATH
```bash
chmod +x enable_servo.sh
./enable_servo.sh
```

### Option 2: Specify Servo path
```bash
chmod +x enable_servo.sh
./enable_servo.sh /your/servo/path
```

### Option 3: Full setup with testing
```bash
chmod +x setup_linux_simple.sh
./setup_linux_simple.sh /your/servo/path
```

## 📝 Manual Configuration

If you prefer to do it manually, just create this file:

**servo_config.toml:**
```toml
[servo]
executable_path = "/your/actual/servo/path"
mode = "real"

[integration]
enable_real_integration = true
use_javascript_injection = true
```

## 🔧 Code Changes

### Current (Simulation):
```rust
use stylo_compute::ServoStyleEngine;
let mut engine = ServoStyleEngine::new()?;  // Simulation mode
```

### Real Servo Integration:
```rust
use stylo_compute::ServoStyleEngineReal;
let mut engine = ServoStyleEngineReal::new()?;  // Real mode
```

## 🧪 Test Commands

```bash
# Test with current simulation engine
cargo run --example servo_integration_demo

# Test with real Servo engine  
cargo run --example real_servo_demo

# Test main application
cargo run
```

## 🎯 That's It!

No need to:
- ❌ Install system dependencies
- ❌ Build Servo from source  
- ❌ Wait 30-60 minutes for compilation

Just:
- ✅ Create config file (30 seconds)
- ✅ Change code to use `ServoStyleEngineReal`
- ✅ Run and enjoy real Stylo integration!

## 🔍 Common Servo Paths

Your Servo might be at:
- `/usr/local/bin/servo`
- `/opt/servo/servo`
- `/home/user/servo/target/debug/servo`
- `/home/user/servo/target/release/servo`
- `$(which servo)` if it's in PATH

Find it with:
```bash
which servo
# or
find /usr /opt /home -name "servo" -type f -executable 2>/dev/null
```
