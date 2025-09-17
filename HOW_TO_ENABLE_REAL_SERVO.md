# 🚀 How to Enable Real Servo Execution

## Current Status
- ✅ **Simulation Mode**: Working (what you have now)
- ❌ **Real Servo Mode**: Not enabled (Servo not installed)

## 🔧 To Enable Real Servo on Linux Machine

### Method 1: Using the Transfer Package (Recommended)

1. **Transfer the project**:
   ```bash
   # The package is ready: stylo-compute-linux.tar.gz
   scp stylo-compute-linux.tar.gz user@linux-machine:/home/user/
   ```

2. **On Linux machine**:
   ```bash
   cd /home/user/
   tar -xzf stylo-compute-linux.tar.gz
   cd stylo-compute-linux-ready/
   ./setup_linux.sh  # This builds Servo and configures everything
   ```

3. **Test real integration**:
   ```bash
   cargo run --example real_servo_demo
   ```

### Method 2: Manual Configuration (If you have Servo elsewhere)

If you already have Servo built on your Linux machine:

1. **Find your Servo executable**:
   ```bash
   find /home -name "servo" -type f -executable 2>/dev/null
   # Or if it's in PATH:
   which servo
   ```

2. **Create configuration file**:
   ```bash
   cat > servo_config.toml << EOF
   [servo]
   executable_path = "/path/to/your/servo/executable"
   mode = "real"

   [integration]
   enable_real_integration = true
   use_javascript_injection = true
   EOF
   ```

3. **Update your code**:
   ```rust
   // Instead of:
   let mut engine = ServoStyleEngine::new()?;  // Simulation mode
   
   // Use:
   let mut engine = ServoStyleEngineReal::new()?;  // Real mode
   
   // Or with custom path:
   let servo_path = Some("/path/to/servo".to_string());
   let mut engine = ServoStyleEngineReal::with_servo_path(servo_path)?;
   ```

## 🎯 Code Changes for Real Integration

### In your selected code `Self::with_servo_path(None)`:

**Current (Simulation)**:
```rust
use stylo_compute::ServoStyleEngine;  // Simulation engine

let mut engine = ServoStyleEngine::new()?;  // Uses simulation
```

**Real Servo Integration**:
```rust
use stylo_compute::ServoStyleEngineReal;  // Real engine

// Method 1: Auto-detect Servo in PATH
let mut engine = ServoStyleEngineReal::new()?;

// Method 2: Custom Servo path
let servo_path = Some("/home/user/servo/target/debug/servo".to_string());
let mut engine = ServoStyleEngineReal::with_servo_path(servo_path)?;

// Method 3: Use None to auto-detect (what you have selected)
let mut engine = ServoStyleEngineReal::with_servo_path(None)?;
```

## 🔍 How to Verify Real Integration is Working

### Expected Output (Real Mode):
```
✅ Servo found - enabling real Stylo integration
   Using Servo from PATH
🚀 Starting Servo process for style computation...
✅ Servo process started successfully
🔄 Querying real Servo process for computed styles...
   Using genuine Stylo APIs via Servo's getComputedStyle()
```

### Expected Output (Simulation Mode):
```
⚠️  Note: Running in simulation mode (Servo executable check disabled)
🔄 Simulating Servo query for computed styles...
```

## 📋 Complete Example for Linux

```rust
use stylo_compute::{ServoStyleEngineReal, ServoStyleError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This will use REAL Servo when available
    let mut engine = ServoStyleEngineReal::with_servo_path(None)?;
    
    engine.set_html(r#"<div class="test">Hello</div>"#)?;
    engine.add_stylesheet(".test { color: red; }")?;
    
    // This calls REAL Stylo APIs through Servo
    let color = engine.get_computed_style(".test", "color").await?;
    println!("Real computed color: {}", color);  // Uses genuine Stylo!
    
    Ok(())
}
```

## 🎉 Summary

- **Current**: Simulation mode (no real Servo needed)
- **Linux Transfer**: Complete package ready with auto-setup
- **Real Integration**: Change `ServoStyleEngine` → `ServoStyleEngineReal`
- **Your Selected Code**: `Self::with_servo_path(None)` will auto-detect Servo in PATH

The transfer package includes everything needed to build Servo and enable real integration on Linux!
