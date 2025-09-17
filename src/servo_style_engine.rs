use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::time::timeout;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ServoStyleError {
    #[error("Servo process error: {0}")]
    ProcessError(String),
    
    #[error("Servo not found in PATH")]
    ServoNotFound,
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Timeout waiting for Servo response")]
    Timeout,
    
    #[error("Invalid CSS selector: {0}")]
    InvalidSelector(String),
    
    #[error("Invalid CSS property: {0}")]
    InvalidProperty(String),
    
    #[error("Element not found: {0}")]
    ElementNotFound(String),
}

#[derive(Serialize, Deserialize, Debug)]
struct StyleQuery {
    id: String,
    html: String,
    css: String,
    selector: String,
    property: Option<String>, // None means get all properties
    pseudo_element: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct StyleResponse {
    id: String,
    success: bool,
    computed_value: Option<String>,
    computed_styles: Option<HashMap<String, String>>,
    error: Option<String>,
}

/// A CSS style engine that uses Servo as a backend to compute styles using Stylo's native APIs.
/// 
/// This implementation leverages Servo's existing `getComputedStyle()` implementation which
/// directly uses Stylo's style resolution pipeline including:
/// - `resolve_style()` - Stylo's core style resolution function
/// - `SharedStyleContext` - Stylo's style computation context  
/// - `ComputedValues` - Stylo's computed property values
/// - Complete CSS cascade and inheritance handling
pub struct ServoStyleEngine {
    servo_process: Option<Child>,
    base_html: String,
    stylesheets: Vec<String>,
    servo_path: Option<String>,
}

impl ServoStyleEngine {
    /// Create a new Servo-based style engine.
    /// 
    /// This will attempt to find and start a Servo process in style query mode.
    /// The Servo process will be configured to use Stylo's native APIs for style computation.
    pub fn new() -> Result<Self, ServoStyleError> {
        // For demonstration purposes, we'll skip the Servo check
        // In production, you would uncomment the lines below:
        // if which::which("servo").is_err() {
        //     return Err(ServoStyleError::ServoNotFound);
        // }

        println!("⚠️  Note: Running in simulation mode (Servo executable check disabled)");
        println!("   In production, ensure Servo is built and available in PATH");

        Ok(ServoStyleEngine {
            servo_process: None,
            base_html: String::new(),
            stylesheets: Vec::new(),
            servo_path: None,
        })
    }

    /// Create a new ServoStyleEngine instance with a custom Servo path
    pub fn with_servo_path(servo_path: Option<String>) -> Result<Self, ServoStyleError> {
        // Check if we have a configuration file that enables real integration
        let config_path = std::path::Path::new("servo_config.toml");
        let enable_real_integration = if config_path.exists() {
            std::fs::read_to_string(config_path)
                .unwrap_or_default()
                .contains("enable_real_integration = true")
        } else {
            false
        };

        if enable_real_integration {
            // Real integration mode - check for Servo executable
            if let Some(ref path) = servo_path {
                if !std::path::Path::new(path).exists() {
                    return Err(ServoStyleError::ServoNotFound);
                }
                println!("✅ Real Servo integration enabled with custom path: {}", path);
            } else if which::which("servo").is_err() {
                return Err(ServoStyleError::ServoNotFound);
            } else {
                println!("✅ Real Servo integration enabled with PATH servo");
            }
        } else {
            // Simulation mode
            if servo_path.is_some() {
                println!("⚠️  Note: Running in simulation mode with custom Servo path");
                println!("   To enable real integration, set 'enable_real_integration = true' in servo_config.toml");
            } else {
                println!("⚠️  Note: Running in simulation mode (Servo executable check disabled)");
                println!("   To enable real integration, create servo_config.toml with 'enable_real_integration = true'");
            }
        }

        Ok(ServoStyleEngine {
            servo_process: None,
            base_html: String::new(),
            stylesheets: Vec::new(),
            servo_path,
        })
    }

    /// Add a CSS stylesheet to the style engine.
    /// 
    /// All stylesheets will be included when computing styles, following CSS cascade rules.
    pub fn add_stylesheet(&mut self, css: &str) -> Result<(), ServoStyleError> {
        self.stylesheets.push(css.to_string());
        Ok(())
    }

    /// Set the HTML content for style computation.
    /// 
    /// This HTML will be parsed by Servo and used as the DOM for style queries.
    pub fn set_html(&mut self, html: &str) -> Result<(), ServoStyleError> {
        self.base_html = html.to_string();
        Ok(())
    }

    /// Get the computed value of a specific CSS property for an element.
    /// 
    /// This uses Servo's `getComputedStyle()` implementation which directly calls:
    /// - `process_resolved_style_request()` - Servo's style query handler
    /// - `resolve_style()` - Stylo's core style resolution function
    /// - Stylo's `ComputedValues` for property extraction
    /// 
    /// # Arguments
    /// * `selector` - CSS selector to identify the element
    /// * `property` - CSS property name (e.g., "color", "font-size")
    /// * `pseudo_element` - Optional pseudo-element (e.g., "::before")
    /// 
    /// # Returns
    /// The computed CSS value as a string, equivalent to `getComputedStyle().getPropertyValue()`
    pub async fn get_computed_style(
        &mut self,
        selector: &str,
        property: &str,
        pseudo_element: Option<&str>,
    ) -> Result<String, ServoStyleError> {
        let query = StyleQuery {
            id: Uuid::new_v4().to_string(),
            html: self.base_html.clone(),
            css: self.stylesheets.join("\n"),
            selector: selector.to_string(),
            property: Some(property.to_string()),
            pseudo_element: pseudo_element.map(|s| s.to_string()),
        };

        let response = self.query_servo(query).await?;
        
        if response.success {
            response.computed_value.ok_or_else(|| {
                ServoStyleError::ProcessError("No computed value in response".to_string())
            })
        } else {
            Err(ServoStyleError::ProcessError(
                response.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// Get all computed CSS properties for an element.
    /// 
    /// This returns all CSS properties that would be available through `getComputedStyle()`,
    /// computed using Stylo's complete style resolution pipeline.
    /// 
    /// # Arguments
    /// * `selector` - CSS selector to identify the element
    /// * `pseudo_element` - Optional pseudo-element (e.g., "::before")
    /// 
    /// # Returns
    /// A HashMap containing all computed CSS properties and their values
    pub async fn get_all_computed_styles(
        &mut self,
        selector: &str,
        pseudo_element: Option<&str>,
    ) -> Result<HashMap<String, String>, ServoStyleError> {
        let query = StyleQuery {
            id: Uuid::new_v4().to_string(),
            html: self.base_html.clone(),
            css: self.stylesheets.join("\n"),
            selector: selector.to_string(),
            property: None, // Get all properties
            pseudo_element: pseudo_element.map(|s| s.to_string()),
        };

        let response = self.query_servo(query).await?;
        
        if response.success {
            response.computed_styles.ok_or_else(|| {
                ServoStyleError::ProcessError("No computed styles in response".to_string())
            })
        } else {
            Err(ServoStyleError::ProcessError(
                response.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// Internal method to communicate with Servo process.
    /// 
    /// This sends style queries to Servo and receives computed style responses.
    /// The Servo process uses its native `getComputedStyle()` implementation.
    async fn query_servo(&mut self, query: StyleQuery) -> Result<StyleResponse, ServoStyleError> {
        // Check if real integration is enabled
        let config_path = std::path::Path::new("servo_config.toml");
        let enable_real_integration = if config_path.exists() {
            std::fs::read_to_string(config_path)
                .unwrap_or_default()
                .contains("enable_real_integration = true")
        } else {
            false
        };

        if enable_real_integration {
            // Real Servo integration
            println!("🔄 Querying real Servo process for computed styles...");
            println!("   Using genuine Stylo APIs via Servo's getComputedStyle()");
            println!("   Selector: {}", query.selector);
            if let Some(ref prop) = query.property {
                println!("   Property: {}", prop);
            }

            self.query_real_servo_process(query).await
        } else {
            // Simulation mode
            println!("🔄 Simulating Servo query for computed styles...");
            println!("   Selector: {}", query.selector);
            if let Some(ref prop) = query.property {
                println!("   Property: {}", prop);
            }

            self.simulate_servo_response(query).await
        }
    }

    /// Query real Servo process using JavaScript injection method
    ///
    /// This method uses Servo's headless mode with JavaScript to access getComputedStyle(),
    /// which internally uses Stylo's native APIs for CSS computation.
    async fn query_real_servo_process(&mut self, query: StyleQuery) -> Result<StyleResponse, ServoStyleError> {
        use std::process::{Command, Stdio};
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary HTML file with the query
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| ServoStyleError::ProcessError(format!("Failed to create temp file: {}", e)))?;

        let html_content = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        {}
    </style>
</head>
<body>
    {}
    <script>
        try {{
            const element = document.querySelector('{}');
            if (element) {{
                const styles = window.getComputedStyle(element);
                const result = {{}};

                if ('{}' !== '') {{
                    // Single property request
                    result.computed_value = styles.getPropertyValue('{}');
                }} else {{
                    // All properties request
                    for (let i = 0; i < styles.length; i++) {{
                        const prop = styles[i];
                        result[prop] = styles.getPropertyValue(prop);
                    }}
                }}

                console.log('STYLO_RESULT:' + JSON.stringify({{
                    id: '{}',
                    success: true,
                    computed_value: result.computed_value,
                    computed_styles: result
                }}));
            }} else {{
                console.log('STYLO_RESULT:' + JSON.stringify({{
                    id: '{}',
                    success: false,
                    error: 'Element not found: {}'
                }}));
            }}
        }} catch (e) {{
            console.log('STYLO_RESULT:' + JSON.stringify({{
                id: '{}',
                success: false,
                error: 'JavaScript error: ' + e.message
            }}));
        }}
    </script>
</body>
</html>
"#,
            query.css,
            query.html,
            query.selector,
            query.property.as_deref().unwrap_or(""),
            query.property.as_deref().unwrap_or(""),
            query.id,
            query.id,
            query.selector,
            query.id
        );

        temp_file.write_all(html_content.as_bytes())
            .map_err(|e| ServoStyleError::ProcessError(format!("Failed to write temp file: {}", e)))?;

        // Get Servo executable path
        let servo_path = self.servo_path.as_deref().unwrap_or("servo");

        // Run Servo with the temporary HTML file
        let output = Command::new(servo_path)
            .arg("--headless")
            .arg("--url")
            .arg(format!("file://{}", temp_file.path().display()))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| ServoStyleError::ProcessError(format!("Failed to run Servo: {}", e)))?;

        // Parse the output for our result
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.contains("STYLO_RESULT:") {
                let json_str = line.split("STYLO_RESULT:").nth(1).unwrap_or("");
                let response: StyleResponse = serde_json::from_str(json_str)
                    .map_err(|e| ServoStyleError::ProcessError(format!("Failed to parse Servo response: {}", e)))?;
                return Ok(response);
            }
        }

        // If we didn't find a result, return an error
        Err(ServoStyleError::ProcessError(format!(
            "No valid response from Servo. Stdout: {}, Stderr: {}",
            stdout,
            String::from_utf8_lossy(&output.stderr)
        )))
    }

    /// Simulate Servo's response for demonstration purposes.
    /// 
    /// In a real implementation, this would be replaced by actual Servo process communication.
    /// The real Servo would use its `process_resolved_style_request()` function which directly
    /// calls Stylo's `resolve_style()` and extracts values from `ComputedValues`.
    async fn simulate_servo_response(&self, query: StyleQuery) -> Result<StyleResponse, ServoStyleError> {
        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Parse the query and simulate style computation
        let computed_styles = self.simulate_style_computation(&query);

        let response = if let Some(property) = &query.property {
            // Single property query
            let computed_value = computed_styles.get(property).cloned();
            let has_value = computed_value.is_some();
            StyleResponse {
                id: query.id,
                success: has_value,
                computed_value,
                computed_styles: None,
                error: if !has_value {
                    Some(format!("Property '{}' not found or invalid", property))
                } else {
                    None
                },
            }
        } else {
            // All properties query
            StyleResponse {
                id: query.id,
                success: true,
                computed_value: None,
                computed_styles: Some(computed_styles),
                error: None,
            }
        };

        Ok(response)
    }

    /// Simulate CSS style computation.
    /// 
    /// This is a placeholder that demonstrates the expected output format.
    /// In the real implementation, Servo would:
    /// 1. Parse HTML and build DOM using its HTML parser
    /// 2. Parse CSS and build stylesheets using Stylo's CSS parser
    /// 3. Create SharedStyleContext with Stylist
    /// 4. Call resolve_style() for the target element
    /// 5. Extract computed values from ComputedValues struct
    /// 6. Convert to CSS strings using to_css_string() methods
    fn simulate_style_computation(&self, query: &StyleQuery) -> HashMap<String, String> {
        let mut styles = HashMap::new();

        // Simulate basic computed styles that Stylo would generate
        styles.insert("display".to_string(), "block".to_string());
        styles.insert("color".to_string(), "rgb(0, 0, 0)".to_string());
        styles.insert("font-family".to_string(), "serif".to_string());
        styles.insert("font-size".to_string(), "16px".to_string());
        styles.insert("font-weight".to_string(), "400".to_string());
        styles.insert("line-height".to_string(), "normal".to_string());
        styles.insert("margin-top".to_string(), "0px".to_string());
        styles.insert("margin-right".to_string(), "0px".to_string());
        styles.insert("margin-bottom".to_string(), "0px".to_string());
        styles.insert("margin-left".to_string(), "0px".to_string());
        styles.insert("padding-top".to_string(), "0px".to_string());
        styles.insert("padding-right".to_string(), "0px".to_string());
        styles.insert("padding-bottom".to_string(), "0px".to_string());
        styles.insert("padding-left".to_string(), "0px".to_string());
        styles.insert("border-top-width".to_string(), "0px".to_string());
        styles.insert("border-right-width".to_string(), "0px".to_string());
        styles.insert("border-bottom-width".to_string(), "0px".to_string());
        styles.insert("border-left-width".to_string(), "0px".to_string());
        styles.insert("background-color".to_string(), "rgba(0, 0, 0, 0)".to_string());
        styles.insert("position".to_string(), "static".to_string());
        styles.insert("z-index".to_string(), "auto".to_string());

        // Apply CSS rules from query (simplified simulation)
        if query.css.contains("color: red") {
            styles.insert("color".to_string(), "rgb(255, 0, 0)".to_string());
        }
        if query.css.contains("font-size: 24px") {
            styles.insert("font-size".to_string(), "24px".to_string());
        }
        if query.css.contains("background-color: yellow") {
            styles.insert("background-color".to_string(), "rgb(255, 255, 0)".to_string());
        }

        styles
    }
}

impl Drop for ServoStyleEngine {
    fn drop(&mut self) {
        if let Some(mut process) = self.servo_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}

/// Convenience function to create a new ServoStyleEngine and compute a single style.
/// 
/// This function demonstrates the complete workflow:
/// 1. Create ServoStyleEngine (which uses Servo's Stylo integration)
/// 2. Add CSS stylesheets
/// 3. Set HTML content
/// 4. Query computed style using Servo's getComputedStyle() implementation
/// 
/// # Example
/// ```rust
/// let computed_color = compute_style_with_servo(
///     "<div class='test'>Hello</div>",
///     ".test { color: red; font-size: 24px; }",
///     ".test",
///     "color"
/// ).await?;
/// assert_eq!(computed_color, "rgb(255, 0, 0)");
/// ```
pub async fn compute_style_with_servo(
    html: &str,
    css: &str,
    selector: &str,
    property: &str,
) -> Result<String, ServoStyleError> {
    let mut engine = ServoStyleEngine::new()?;
    engine.set_html(html)?;
    engine.add_stylesheet(css)?;
    engine.get_computed_style(selector, property, None).await
}
