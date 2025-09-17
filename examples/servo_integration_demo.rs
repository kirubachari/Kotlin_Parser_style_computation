use stylo_compute::{ServoStyleEngine, ServoStyleError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Servo-Stylo Integration Demonstration");
    println!("=========================================");
    println!("This demonstrates how to use Servo as an intermediary to access Stylo's native APIs");
    println!();

    // Test 1: Basic engine creation
    println!("📋 Test 1: Creating ServoStyleEngine");
    let mut engine = ServoStyleEngine::new()?;
    println!("✅ Successfully created ServoStyleEngine");
    println!("   This engine communicates with Servo processes to access Stylo's APIs");
    println!();

    // Test 2: Setting HTML content
    println!("📋 Test 2: Setting HTML content");
    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Stylo Test</title>
        </head>
        <body>
            <div class="container" id="main">
                <h1 class="title">Hello, Stylo!</h1>
                <p class="content highlight">This is a test paragraph.</p>
                <span class="small-text">Small text here</span>
            </div>
        </body>
        </html>
    "#;
    
    engine.set_html(html_content)?;
    println!("✅ HTML content set successfully");
    println!("   Content includes various elements with classes and IDs for testing");
    println!();

    // Test 3: Adding CSS stylesheets
    println!("📋 Test 3: Adding CSS stylesheets");
    let css_content = r#"
        .container {
            width: 800px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
            border: 1px solid #ddd;
        }
        
        .title {
            color: #333;
            font-size: 32px;
            font-weight: bold;
            margin-bottom: 16px;
            text-align: center;
        }
        
        .content {
            font-size: 18px;
            line-height: 1.6;
            color: #666;
            margin-bottom: 12px;
        }
        
        .highlight {
            background-color: yellow;
            padding: 8px;
            border-left: 4px solid orange;
        }
        
        .small-text {
            font-size: 14px;
            color: #999;
            font-style: italic;
        }
        
        #main {
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
    "#;
    
    engine.add_stylesheet(css_content)?;
    println!("✅ CSS stylesheet added successfully");
    println!("   Stylesheet includes complex selectors and various CSS properties");
    println!();

    // Test 4: Demonstrate the API calls (these will show simulation results)
    println!("📋 Test 4: Demonstrating Servo-Stylo API calls");
    println!("Note: This shows simulated responses. In a real implementation,");
    println!("these would call Servo's actual getComputedStyle() implementation.");
    println!();

    // Test individual property queries
    let test_queries = vec![
        (".title", "color"),
        (".title", "font-size"),
        (".title", "font-weight"),
        (".content", "background-color"),
        (".highlight", "padding"),
        ("#main", "box-shadow"),
    ];

    for (selector, property) in test_queries {
        match engine.get_computed_style(selector, property, None).await {
            Ok(value) => {
                println!("  {} -> {}: {}", selector, property, value);
            }
            Err(ServoStyleError::ElementNotFound(msg)) => {
                println!("  {} -> {}: Element not found ({})", selector, property, msg);
            }
            Err(e) => {
                println!("  {} -> {}: Error - {}", selector, property, e);
            }
        }
    }
    println!();

    // Test 5: Get all computed styles for an element
    println!("📋 Test 5: Getting all computed styles for .title element");
    match engine.get_all_computed_styles(".title", None).await {
        Ok(styles) => {
            println!("✅ Retrieved {} computed properties:", styles.len());
            
            // Show key properties
            let key_props = ["display", "color", "font-family", "font-size", "font-weight", 
                           "text-align", "margin-bottom", "background-color"];
            
            for prop in key_props {
                if let Some(value) = styles.get(prop) {
                    println!("    {}: {}", prop, value);
                }
            }
            
            if styles.len() > key_props.len() {
                println!("    ... and {} more properties", styles.len() - key_props.len());
            }
        }
        Err(e) => {
            println!("❌ Error getting all styles: {}", e);
        }
    }
    println!();

    // Test 6: Show the underlying Servo-Stylo integration
    println!("📋 Test 6: Understanding the Servo-Stylo Integration");
    println!("This implementation demonstrates how Servo acts as an intermediary to Stylo:");
    println!();
    println!("🔄 API Call Flow:");
    println!("   1. Your Application");
    println!("   2. ↓ ServoStyleEngine.get_computed_style()");
    println!("   3. ↓ Servo Process Communication");
    println!("   4. ↓ Servo's window.getComputedStyle() implementation");
    println!("   5. ↓ process_resolved_style_request() - Servo's style handler");
    println!("   6. ↓ resolve_style() - Stylo's CORE function");
    println!("   7. ↓ SharedStyleContext - Stylo's computation context");
    println!("   8. ↓ ComputedValues - Stylo's native computed properties");
    println!("   9. ↑ Return computed CSS values");
    println!();
    println!("🎯 Key Benefits:");
    println!("   ✅ Uses genuine Stylo APIs (not custom implementation)");
    println!("   ✅ Leverages Servo's complete DOM trait implementations");
    println!("   ✅ Gets production-quality CSS engine behavior");
    println!("   ✅ Avoids implementing 25,000+ lines of DOM integration code");
    println!("   ✅ Maintains compatibility with web standards");
    println!();
    println!("🔧 Implementation Status:");
    println!("   ✅ ServoStyleEngine API - Complete");
    println!("   ✅ Error handling and validation - Complete");
    println!("   ✅ CSS and HTML content management - Complete");
    println!("   ⚠️  Servo process communication - Simulated (needs real Servo integration)");
    println!("   ⚠️  JSON protocol implementation - Defined (needs Servo modifications)");
    println!();

    println!("🎉 Servo-Stylo Integration Demonstration Complete!");
    println!("This approach provides access to Stylo's native APIs through Servo's existing integration.");
    
    Ok(())
}
