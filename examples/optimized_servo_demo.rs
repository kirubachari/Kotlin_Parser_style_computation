use stylo_compute::ServoStyleEngineOptimized;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Servo Style Engine - Optimized Demo");
    println!("=====================================");

    // Read Servo path from config if available
    let servo_path = read_servo_config().unwrap_or_else(|| {
        println!("⚠️  No servo_config.toml found. Please run ./enable_servo.sh first");
        "/home/test/clone/servo/target/debug/servo".to_string()
    });

    // Create optimized engine with daemon mode and batch processing
    let mut engine = ServoStyleEngineOptimized::with_options(
        Some(servo_path.clone()), 
        true,  // Use daemon mode
        5      // Batch size
    )?;

    // Test cases
    println!("\n📋 Running optimized test cases...");

    // Test 1: Single property with optimized engine
    println!("\n🧪 Test 1: Single Property (Optimized)");
    engine.set_html("<div style='color: red;'>Hello World</div>")?;
    engine.add_stylesheet("/* Additional styles */")?;
    
    match engine.get_computed_style("div", "color").await {
        Ok(color) => println!("   ✅ div -> color: {}", color),
        Err(e) => println!("   ❌ Error: {}", e),
    }

    // Test 2: Batch processing multiple queries
    println!("\n🧪 Test 2: Batch Processing");
    engine.set_html(r#"
        <div class="red-text">Red text</div>
        <p class="large-text">Large paragraph</p>
        <span id="special">Special span</span>
    "#)?;
    engine.add_stylesheet(r#"
        .red-text { color: red; font-weight: bold; }
        .large-text { font-size: 20px; color: blue; }
        #special { background-color: yellow; padding: 10px; }
    "#)?;

    let batch_requests = vec![
        ("div".to_string(), Some("color".to_string())),
        ("p".to_string(), Some("font-size".to_string())),
        ("#special".to_string(), Some("background-color".to_string())),
        ("div".to_string(), Some("font-weight".to_string())),
    ];

    println!("   Processing {} queries in batch...", batch_requests.len());
    match engine.compute_styles_batch(batch_requests).await {
        Ok(results) => {
            for (selector, result) in results {
                match result {
                    Ok(value) => println!("   ✅ {} -> {}", selector, value),
                    Err(e) => println!("   ❌ {} -> Error: {}", selector, e),
                }
            }
        }
        Err(e) => println!("   ❌ Batch error: {}", e),
    }

    // Test 3: All computed styles for an element
    println!("\n🧪 Test 3: All Computed Styles");
    match engine.get_all_computed_styles("div").await {
        Ok(styles) => {
            println!("   ✅ Found {} computed properties for div", styles.len());
            
            // Show some key properties
            let key_props = ["color", "font-size", "font-weight", "display", "margin"];
            for prop in &key_props {
                if let Some(value) = styles.get(*prop) {
                    println!("   📋   {}: {}", prop, value);
                }
            }
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }

    // Test 4: Convenience function for batch processing
    println!("\n🧪 Test 4: Convenience Batch Function");
    let batch_queries = vec![
        ("h1".to_string(), "color".to_string(), None),
        ("p".to_string(), "font-size".to_string(), None),
        ("div".to_string(), "display".to_string(), None),
    ];

    let html = "<h1>Title</h1><p>Paragraph</p><div>Content</div>";
    let css = "h1 { color: green; } p { font-size: 16px; } div { display: block; }";

    match stylo_compute::compute_styles_batch_optimized(
        html, 
        css, 
        batch_queries, 
        Some(servo_path.clone())
    ).await {
        Ok(results) => {
            println!("   ✅ Batch convenience function results:");
            for (selector, result) in results {
                match result {
                    Ok(value) => println!("   📋 {} -> {}", selector, value),
                    Err(e) => println!("   ❌ {} -> Error: {}", selector, e),
                }
            }
        }
        Err(e) => println!("   ❌ Convenience function error: {}", e),
    }

    println!("\n🎉 Optimized demo completed!");
    println!("📁 Check /tmp/ for debug files and batch results");

    Ok(())
}

fn read_servo_config() -> Option<String> {
    use std::fs;
    let config_content = fs::read_to_string("servo_config.toml").ok()?;
    
    for line in config_content.lines() {
        if line.starts_with("servo_path") {
            if let Some(path) = line.split('=').nth(1) {
                return Some(path.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}