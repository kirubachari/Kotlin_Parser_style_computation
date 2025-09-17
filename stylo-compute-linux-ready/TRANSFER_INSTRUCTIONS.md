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
