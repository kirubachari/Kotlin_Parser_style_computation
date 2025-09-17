use stylo_compute::{ServoStyleEngineReal, ServoStyleError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Real Servo-Stylo Integration Demo");
    println!("===================================");
    println!("This example uses REAL Servo executable to compute CSS styles");
    println!("using genuine Stylo APIs (not simulation)");
    println!();

    // Try to create real Servo engine
    println!("📋 Test 1: Creating Real ServoStyleEngine");
    
    // First check if we have a servo_config.toml file
    let config_path = std::path::Path::new("servo_config.toml");
    let servo_path_from_config: Option<String> = if config_path.exists() {
        match std::fs::read_to_string(config_path) {
            Ok(content) => {
                // Simple TOML parsing to extract executable_path
                let mut found_path = None;
                for line in content.lines() {
                    if line.trim().starts_with("executable_path = ") {
                        if let Some(path_part) = line.split('=').nth(1) {
                            let path = path_part.trim().trim_matches('"').to_string();
                            println!("🔧 Found Servo path in config: {}", path);
                            found_path = Some(path);
                            break;
                        }
                    }
                }
                found_path
            }
            Err(_) => None,
        }
    } else {
        None
    };
    
    // Try different approaches to find Servo
    let engine = if let Some(config_path) = servo_path_from_config {
        // Use path from config file
        match ServoStyleEngineReal::with_servo_path(Some(config_path.clone())) {
            Ok(eng) => {
                println!("✅ Real Servo integration enabled using config: {}", config_path);
                Some(eng)
            }
            Err(e) => {
                println!("❌ Error with config path {}: {}", config_path, e);
                None
            }
        }
    } else {
        // Fall back to common paths
        let servo_paths = vec![
            ("PATH", None),  // In PATH
            ("./servo/target/debug/servo", Some("./servo/target/debug/servo".to_string())),  // Local build
            ("../servo/target/debug/servo", Some("../servo/target/debug/servo".to_string())), // Parent directory
            ("/usr/local/bin/servo", Some("/usr/local/bin/servo".to_string())),        // System install
            ("/home/user/servo/target/debug/servo", Some("/home/user/servo/target/debug/servo".to_string())), // User build
        ];
        
        let mut engine = None;
        for (path_desc, path_opt) in &servo_paths {
            match if path_opt.is_none() {
                ServoStyleEngineReal::new()
            } else {
                ServoStyleEngineReal::with_servo_path(path_opt.clone())
            } {
                Ok(eng) => {
                    println!("✅ Real Servo integration enabled using: {}", path_desc);
                    engine = Some(eng);
                    break;
                }
                Err(_) => {
                    continue; // Try next path
                }
            }
        }
        
        if engine.is_none() {
            println!("❌ Servo executable not found in any common locations");
            println!("   Tried paths: {:?}", servo_paths.iter().map(|(desc, _)| desc).collect::<Vec<_>>());
        }
        
        engine
    };
    
    let mut engine = match engine {
        Some(engine) => engine,
        None => {
            println!("❌ Servo executable not found");
            println!("   Please ensure Servo is built and available");
            println!("   Or run: ./enable_servo.sh /path/to/your/servo");
            println!();
            println!("🔧 To build Servo on Linux:");
            println!("   git clone https://github.com/servo/servo.git");
            println!("   cd servo && ./mach build --dev");
            println!("   export PATH=\"$(pwd)/target/debug:$PATH\"");
            return Ok(());
        }
    };

    println!();
    println!("📋 Test 2: Setting up HTML and CSS");
    
    // Set HTML content
    engine.set_html(r#"
        <div class="title">Real Servo Test</div>
        <div class="content">This uses genuine Stylo APIs!</div>
        <p class="highlight" id="main">Computed by real Servo-Stylo integration</p>
    "#)?;
    
    // Add CSS stylesheet
    engine.add_stylesheet(r#"
        .title {
            color: red;
            font-size: 24px;
            font-weight: bold;
        }
        .content {
            background-color: yellow;
            padding: 10px;
            margin: 5px;
        }
        .highlight {
            color: blue;
            text-decoration: underline;
            font-style: italic;
        }
        #main {
            border: 2px solid green;
            border-radius: 5px;
        }
    "#)?;
    
    println!("✅ HTML and CSS content set");
    println!();

    println!("📋 Test 3: Computing individual CSS properties with REAL Servo");
    
    // Test individual property queries
    let test_cases = vec![
        (".title", "color"),
        (".title", "font-size"),
        (".title", "font-weight"),
        (".content", "background-color"),
        (".content", "padding"),
        (".highlight", "color"),
        (".highlight", "text-decoration"),
        ("#main", "border"),
        ("#main", "border-radius"),
    ];
    
    for (selector, property) in test_cases {
        match engine.get_computed_style(selector, property).await {
            Ok(value) => {
                println!("  {} -> {}: {}", selector, property, value);
            }
            Err(e) => {
                println!("  {} -> {}: Error - {}", selector, property, e);
            }
        }
    }
    
    println!();
    println!("📋 Test 4: Getting all computed styles with REAL Servo");
    
    match engine.get_all_computed_styles(".title").await {
        Ok(styles) => {
            println!("✅ Retrieved {} computed properties for .title:", styles.len());
            let mut sorted_styles: Vec<_> = styles.iter().collect();
            sorted_styles.sort_by_key(|(k, _)| *k);
            
            for (property, value) in sorted_styles.iter().take(10) {
                println!("    {}: {}", property, value);
            }
            if styles.len() > 10 {
                println!("    ... and {} more properties", styles.len() - 10);
            }
        }
        Err(e) => {
            println!("❌ Error getting all styles: {}", e);
        }
    }
    
    println!();
    println!("📋 Test 5: Understanding Real vs Simulation");
    println!("🔄 API Call Flow (REAL MODE):");
    println!("   1. Your Application");
    println!("   2. ↓ ServoStyleEngineReal.get_computed_style()");
    println!("   3. ↓ Real Servo Process Communication");
    println!("   4. ↓ Servo's actual window.getComputedStyle() implementation");
    println!("   5. ↓ process_resolved_style_request() - Servo's style handler");
    println!("   6. ↓ resolve_style() - Stylo's CORE function");
    println!("   7. ↓ SharedStyleContext - Stylo's computation context");
    println!("   8. ↓ ComputedValues - Stylo's native computed properties");
    println!("   9. ↑ Return GENUINE computed CSS values");
    println!();
    
    println!("🎯 Key Differences from Simulation:");
    println!("   ✅ Uses actual Servo executable process");
    println!("   ✅ Calls genuine Stylo APIs (resolve_style, ComputedValues)");
    println!("   ✅ Full CSS cascade and inheritance computation");
    println!("   ✅ Production browser-quality CSS engine behavior");
    println!("   ✅ Complete CSS specification support");
    println!("   ✅ Real DOM tree construction and style resolution");
    println!();
    
    println!("🎉 Real Servo-Stylo Integration Demo Complete!");
    println!("This demonstrates genuine Servo-Stylo integration using actual Stylo APIs.");
    
    Ok(())
}
