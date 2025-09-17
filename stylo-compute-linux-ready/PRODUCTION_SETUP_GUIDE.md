# Production Setup Guide - Real Servo-Stylo Integration

## 🎯 **Steps for Production Use**

### Step 1: Build Servo with Style Query Support

```bash
# 1. Navigate to the servo directory
cd servo

# 2. Install Servo build dependencies
# On macOS:
brew install cmake pkg-config freetype fontconfig

# On Ubuntu/Debian:
sudo apt-get install cmake pkg-config libfreetype6-dev libfontconfig1-dev

# 3. Build Servo (this takes 30-60 minutes)
./mach build --release

# 4. Verify the build
ls target/release/servo  # Should exist
```

### Step 2: Add Style Query Mode to Servo

The current Servo doesn't have a built-in style query mode. You would need to:

```rust
// Add to servo/components/servo/main.rs
fn handle_style_query_mode() {
    // Read JSON queries from stdin
    // Process using existing getComputedStyle implementation
    // Return JSON responses to stdout
}
```

**Alternative**: Use Servo's existing headless mode and inject JavaScript:

```bash
servo --headless --url "data:text/html,<html>...</html>" \
      --script "console.log(JSON.stringify(window.getComputedStyle(...)))"
```

### Step 3: Update ServoStyleEngine Implementation

```rust
// In src/servo_style_engine.rs
pub fn new() -> Result<Self, ServoStyleError> {
    // Re-enable Servo check for production
    if which::which("servo").is_err() {
        return Err(ServoStyleError::ServoNotFound);
    }

    Ok(ServoStyleEngine {
        servo_process: None,
        base_html: String::new(),
        stylesheets: Vec::new(),
    })
}

async fn query_servo_process(&mut self, query: StyleQuery) -> Result<StyleResponse, ServoStyleError> {
    // Replace simulation with real Servo process communication
    
    // 1. Start Servo process if not running
    if self.servo_process.is_none() {
        let child = Command::new("servo")
            .arg("--headless")
            .arg("--style-query-mode")  // Custom flag you'd add
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        self.servo_process = Some(child);
    }

    // 2. Send JSON query to Servo
    let query_json = serde_json::to_string(&query)?;
    if let Some(process) = &mut self.servo_process {
        process.stdin.as_mut().unwrap()
            .write_all(query_json.as_bytes())?;
    }

    // 3. Read JSON response from Servo
    let mut response_buffer = String::new();
    if let Some(process) = &mut self.servo_process {
        process.stdout.as_mut().unwrap()
            .read_to_string(&mut response_buffer)?;
    }

    // 4. Parse and return response
    let response: StyleResponse = serde_json::from_str(&response_buffer)?;
    Ok(response)
}
```

### Step 4: Environment Setup

```bash
# 1. Add Servo to PATH
export PATH="$PATH:/path/to/servo/target/release"

# 2. Verify installation
servo --version

# 3. Test style query mode (after implementing)
echo '{"html":"<div>test</div>","css":"div{color:red}","selector":"div","property":"color"}' | \
servo --headless --style-query-mode
```

### Step 5: Update Dependencies

```toml
# In Cargo.toml - add for real process communication
[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
which = "4.0"
thiserror = "1.0"
# Remove tempfile, uuid if not needed for real implementation
```

---

## 🔧 **Implementation Complexity**

### What You Get:
- ✅ **Genuine Stylo APIs** - Real `resolve_style()` calls
- ✅ **Production CSS Engine** - Same as Firefox browser
- ✅ **Full CSS Support** - Complete specification compliance
- ✅ **Web Standards** - Exact `getComputedStyle()` behavior

### What You Need to Build:
1. **Servo Style Query Mode** - Custom Servo modification (~500-1000 lines)
2. **Process Communication** - Replace simulation with real IPC (~200 lines)
3. **Error Handling** - Robust process management (~100 lines)
4. **Performance Optimization** - Connection pooling, caching (~300 lines)

**Total Effort**: ~1-2 weeks of development + Servo build time

---

## 🚀 **Alternative Approaches**

### Option A: Servo JavaScript Injection
```bash
# Use existing Servo with JavaScript evaluation
servo --headless --url "data:text/html,$HTML" \
      --script "console.log(JSON.stringify(getComputedStyle($SELECTOR)))"
```

### Option B: WebDriver + Headless Browser
```rust
// Use existing browser automation
let driver = WebDriver::new("http://localhost:4444")?;
driver.goto("data:text/html,...")?;
let styles = driver.execute_script("return getComputedStyle(...)")?;
```

### Option C: Direct Stylo Integration
```rust
// Implement the 25,000+ lines of DOM traits yourself
// (Not recommended - extremely complex)
```

---

## 📊 **Recommendation**

**For Learning/Prototyping**: Use current simulation mode - it demonstrates the complete API and architecture.

**For Production**: 
1. **Start with WebDriver approach** (Option B) - easier to implement, still uses real CSS engines
2. **Upgrade to custom Servo** (Steps 1-5) when you need maximum performance and control
3. **Avoid direct Stylo integration** unless you're building a browser engine

The current implementation provides the perfect foundation - the API is designed correctly, and switching from simulation to real Servo is a straightforward replacement of the `simulate_servo_response()` method.
