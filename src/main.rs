use stylo_compute::{ServoStyleEngineReal, compute_style_with_servo_real};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Servo-based CSS Style Engine");
    println!("================================");
    println!("Using Servo's getComputedStyle() implementation with Stylo's native APIs");

    // Example 1: Using the convenience function
    println!("\n📋 Example 1: Single property query");
    let computed_color = compute_style_with_servo_real(
        r#"<div class="highlight" id="main">Hello, World!</div>"#,
        r#".highlight { color: red; font-size: 24px; background-color: yellow; font-weight: bold; }"#,
        ".highlight",
        "color",
        None  // Use default servo path
    ).await?;
    println!("✅ Computed color: {}", computed_color);

    // Example 2: Using the full engine API
    println!("\n📋 Example 2: Full style engine");
    let mut engine = ServoStyleEngineReal::new()?;
    println!("✅ Created Servo-based style engine");

    // Set HTML content
    engine.set_html(r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test</title></head>
        <body>
            <div class="highlight" id="main">
                <p class="content">Hello, World!</p>
                <span class="small">Small text</span>
            </div>
        </body>
        </html>
    "#)?;
    println!("✅ Set HTML content");

    // Add CSS stylesheets
    engine.add_stylesheet(r#"
        .highlight {
            color: red;
            font-size: 24px;
            background-color: yellow;
            font-weight: bold;
            margin: 10px;
            padding: 15px;
        }
        .content {
            font-size: 18px;
            line-height: 1.5;
            margin-bottom: 8px;
        }
        .small {
            font-size: 12px;
            color: gray;
        }
    "#)?;
    println!("✅ Added CSS stylesheets");

    // Get computed style for specific properties
    println!("\n🔍 Querying computed styles using Servo's getComputedStyle()...");

    let color = engine.get_computed_style(".highlight", "color").await?;
    println!("  color: {}", color);

    let font_size = engine.get_computed_style(".highlight", "font-size").await?;
    println!("  font-size: {}", font_size);

    let background = engine.get_computed_style(".highlight", "background-color").await?;
    println!("  background-color: {}", background);

    // Get all computed styles for an element
    println!("\n📊 Getting all computed styles for .highlight element:");
    let all_styles = engine.get_all_computed_styles(".highlight").await?;

    // Display key computed styles
    let key_properties = ["display", "color", "font-family", "font-size", "font-weight",
                         "background-color", "margin-top", "padding-top", "border-top-width"];

    for property in key_properties {
        if let Some(value) = all_styles.get(property) {
            println!("  {}: {}", property, value);
        }
    }

    println!("\n🎯 Servo-based style computation completed successfully!");
    println!("   This used Servo's native getComputedStyle() implementation");
    println!("   which directly calls Stylo's style resolution APIs:");
    println!("   • process_resolved_style_request()");
    println!("   • resolve_style() - Stylo's core function");
    println!("   • SharedStyleContext - Stylo's computation context");
    println!("   • ComputedValues - Stylo's computed property values");

    Ok(())
}



