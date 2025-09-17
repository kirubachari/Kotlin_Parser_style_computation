// Example: How to enable real Servo execution in your code

use crate::servo_style_engine_real::ServoStyleEngineReal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Method 1: Use Servo from PATH (if installed globally)
    let mut engine = ServoStyleEngineReal::new()?;
    
    // Method 2: Use Servo from custom path
    let servo_path = Some("/path/to/your/servo/executable".to_string());
    let mut engine = ServoStyleEngineReal::with_servo_path(servo_path)?;
    
    // Method 3: Use Servo from your built version
    let servo_path = Some("/Users/kiruba-2957/Development/Kotlin-Parser-Rust/servo/target/debug/servo".to_string());
    let mut engine = ServoStyleEngineReal::with_servo_path(servo_path)?;
    
    // Set up HTML and CSS
    engine.set_html(r#"
        <div class="title">Hello World</div>
        <div class="content">Content here</div>
    "#)?;
    
    engine.add_stylesheet(r#"
        .title { color: red; font-size: 24px; }
        .content { background-color: yellow; }
    "#)?;
    
    // Get computed styles using REAL Servo-Stylo integration
    let color = engine.get_computed_style(".title", "color").await?;
    println!("Real computed color: {}", color);
    
    let all_styles = engine.get_all_computed_styles(".title").await?;
    println!("All computed styles: {:?}", all_styles);
    
    Ok(())
}
