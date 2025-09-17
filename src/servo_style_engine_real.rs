use std::collections::HashMap;
use std::process::Command;
use std::fs;
use std::io::Write;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tempfile::NamedTempFile;

#[derive(Error, Debug)]
pub enum ServoStyleError {
    #[error("Servo executable not found")]
    ServoNotFound,
    #[error("Failed to start Servo process: {0}")]
    ProcessStartError(#[from] std::io::Error),
    #[error("Servo process communication error: {0}")]
    CommunicationError(String),
    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Style computation failed: {0}")]
    ComputationError(String),
}

#[derive(Serialize, Deserialize, Debug)]
struct StyleQuery {
    id: String,
    html: String,
    css: String,
    selector: String,
    property: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct StyleResponse {
    id: String,
    success: bool,
    computed_value: Option<String>,
    computed_styles: Option<HashMap<String, String>>,
    error: Option<String>,
}

/// Real Servo-based CSS style engine that uses Stylo's native APIs
/// 
/// This implementation creates HTML files with embedded JavaScript to extract computed styles,
/// then runs Servo to process them and extract the results using real Stylo APIs.
pub struct ServoStyleEngineReal {
    base_html: String,
    stylesheets: Vec<String>,
    servo_path: Option<String>,
}

impl ServoStyleEngineReal {
    /// Create a new ServoStyleEngine instance with real Servo integration
    pub fn new() -> Result<Self, ServoStyleError> {
        Self::with_servo_path(None)
    }

    /// Create a new ServoStyleEngine instance with a custom Servo path
    pub fn with_servo_path(servo_path: Option<String>) -> Result<Self, ServoStyleError> {
        // Check if Servo is available
        if let Some(ref path) = servo_path {
            if !std::path::Path::new(path).exists() {
                return Err(ServoStyleError::ServoNotFound);
            }
        } else if which::which("servo").is_err() {
            return Err(ServoStyleError::ServoNotFound);
        }
        
        println!("✅ Servo found - enabling real Stylo integration");
        if let Some(ref path) = servo_path {
            println!("   Using custom Servo path: {}", path);
        } else {
            println!("   Using Servo from PATH");
        }

        Ok(ServoStyleEngineReal {
            base_html: String::new(),
            stylesheets: Vec::new(),
            servo_path,
        })
    }

    /// Add a CSS stylesheet to the style engine
    pub fn add_stylesheet(&mut self, css: &str) -> Result<(), ServoStyleError> {
        self.stylesheets.push(css.to_string());
        Ok(())
    }

    /// Set the HTML content for style computation
    pub fn set_html(&mut self, html: &str) -> Result<(), ServoStyleError> {
        self.base_html = html.to_string();
        Ok(())
    }

    /// Create an HTML file with embedded JavaScript to extract computed styles
    fn create_style_extraction_html(&self, selector: &str, property: Option<&str>) -> String {
        let combined_css = self.stylesheets.join("\n");
        
        let script = if let Some(prop) = property {
            format!(r#"
                window.addEventListener('load', function() {{
                    try {{
                        var element = document.querySelector('{}');
                        if (element) {{
                            var computedStyle = window.getComputedStyle(element);
                            var value = computedStyle.getPropertyValue('{}');
                            console.log('COMPUTED_STYLE_RESULT:' + JSON.stringify({{
                                selector: '{}',
                                property: '{}',
                                value: value
                            }}));
                        }} else {{
                            console.log('COMPUTED_STYLE_ERROR:Element not found');
                        }}
                    }} catch (e) {{
                        console.log('COMPUTED_STYLE_ERROR:' + e.message);
                    }}
                    // Give Servo time to log then exit
                    setTimeout(function() {{ window.close(); }}, 100);
                }});
            "#, selector, prop, selector, prop)
        } else {
            format!(r#"
                window.addEventListener('load', function() {{
                    try {{
                        var element = document.querySelector('{}');
                        if (element) {{
                            var computedStyle = window.getComputedStyle(element);
                            var styles = {{}};
                            for (var i = 0; i < computedStyle.length; i++) {{
                                var propName = computedStyle[i];
                                styles[propName] = computedStyle.getPropertyValue(propName);
                            }}
                            console.log('COMPUTED_STYLES_RESULT:' + JSON.stringify({{
                                selector: '{}',
                                styles: styles
                            }}));
                        }} else {{
                            console.log('COMPUTED_STYLE_ERROR:Element not found');
                        }}
                    }} catch (e) {{
                        console.log('COMPUTED_STYLE_ERROR:' + e.message);
                    }}
                    setTimeout(function() {{ window.close(); }}, 100);
                }});
            "#, selector, selector)
        };

        format!(r#"<!DOCTYPE html>
<html>
<head>
    <style>
        {}
    </style>
</head>
<body>
    {}
    <script>
        {}
    </script>
</body>
</html>"#, combined_css, self.base_html, script)
    }

    /// Run Servo with the HTML file and extract computed styles from output
    async fn run_servo_and_extract_styles(&self, html_content: &str) -> Result<String, ServoStyleError> {
        // Create temporary HTML file
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| ServoStyleError::CommunicationError(format!("Failed to create temp file: {}", e)))?;
        
        temp_file.write_all(html_content.as_bytes())
            .map_err(|e| ServoStyleError::CommunicationError(format!("Failed to write temp file: {}", e)))?;
        
        let temp_path = temp_file.path();
        let servo_cmd = self.servo_path.as_deref().unwrap_or("servo");
        
        println!("🚀 Running Servo to compute styles using real Stylo APIs...");
        println!("   HTML file: {:?}", temp_path);
        println!("   Servo command: {}", servo_cmd);
        
        // Run Servo with timeout and better arguments
        let output = Command::new(servo_cmd)
            .arg("--headless")
            .arg("--disable-crash-reporter")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--virtual-time-budget=5000")  // 5 second timeout
            .arg(format!("file://{}", temp_path.display()))
            .output()
            .map_err(|e| ServoStyleError::CommunicationError(format!("Failed to run Servo: {}", e)))?;
        
        println!("   Servo exit status: {}", output.status);
        
        // Extract computed style results from Servo's console output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        println!("   Servo stdout: {}", stdout);
        println!("   Servo stderr: {}", stderr);
        
        // Look for our computed style results in the output
        for line in stdout.lines().chain(stderr.lines()) {
            if line.contains("COMPUTED_STYLE_RESULT:") {
                if let Some(json_part) = line.split("COMPUTED_STYLE_RESULT:").nth(1) {
                    println!("   Found result: {}", json_part);
                    return Ok(json_part.to_string());
                }
            }
            if line.contains("COMPUTED_STYLES_RESULT:") {
                if let Some(json_part) = line.split("COMPUTED_STYLES_RESULT:").nth(1) {
                    println!("   Found all styles result: {}", json_part);
                    return Ok(json_part.to_string());
                }
            }
            if line.contains("COMPUTED_STYLE_ERROR:") {
                if let Some(error_part) = line.split("COMPUTED_STYLE_ERROR:").nth(1) {
                    return Err(ServoStyleError::CommunicationError(format!("Servo error: {}", error_part)));
                }
            }
        }
        
        Err(ServoStyleError::CommunicationError(format!(
            "No computed style result found in Servo output. Stdout: {}, Stderr: {}", 
            stdout, stderr
        )))
    }

    /// Query Servo process for computed styles using real Stylo APIs
    async fn query_servo_process(&mut self, query: StyleQuery) -> Result<StyleResponse, ServoStyleError> {
        println!("🔄 Querying real Servo process for computed styles...");
        println!("   Using genuine Stylo APIs via Servo's getComputedStyle()");
        
        let html_content = self.create_style_extraction_html(
            &query.selector, 
            query.property.as_deref()
        );
        
        let result_json = self.run_servo_and_extract_styles(&html_content).await?;
        
        // Parse the JSON result
        if query.property.is_some() {
            // Single property result
            #[derive(Deserialize)]
            struct SingleResult {
                value: String,
            }
            
            let result: SingleResult = serde_json::from_str(&result_json)
                .map_err(|e| ServoStyleError::CommunicationError(format!("JSON parse error: {}", e)))?;
            
            Ok(StyleResponse {
                id: query.id,
                success: true,
                computed_value: Some(result.value),
                computed_styles: None,
                error: None,
            })
        } else {
            // All styles result
            #[derive(Deserialize)]
            struct AllStylesResult {
                styles: HashMap<String, String>,
            }
            
            let result: AllStylesResult = serde_json::from_str(&result_json)
                .map_err(|e| ServoStyleError::CommunicationError(format!("JSON parse error: {}", e)))?;
            
            Ok(StyleResponse {
                id: query.id,
                success: true,
                computed_value: None,
                computed_styles: Some(result.styles),
                error: None,
            })
        }
    }

    /// Get computed style for a specific CSS property using real Stylo APIs
    /// 
    /// This method sends a query to Servo, which then:
    /// 1. Parses the HTML and CSS using Servo's DOM implementation
    /// 2. Calls window.getComputedStyle() implementation
    /// 3. Invokes process_resolved_style_request()
    /// 4. Executes Stylo's resolve_style() - THE CORE STYLO FUNCTION
    /// 5. Uses SharedStyleContext and ComputedValues from Stylo
    /// 6. Returns genuine computed CSS values
    pub async fn get_computed_style(&mut self, selector: &str, property: &str) -> Result<String, ServoStyleError> {
        let combined_css = self.stylesheets.join("\n");
        
        let query = StyleQuery {
            id: uuid::Uuid::new_v4().to_string(),
            html: self.base_html.clone(),
            css: combined_css,
            selector: selector.to_string(),
            property: Some(property.to_string()),
        };

        let response = self.query_servo_process(query).await?;
        
        if response.success {
            response.computed_value.ok_or_else(|| {
                ServoStyleError::ComputationError("No computed value returned".to_string())
            })
        } else {
            Err(ServoStyleError::ComputationError(
                response.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// Get all computed styles for an element using real Stylo APIs
    pub async fn get_all_computed_styles(&mut self, selector: &str) -> Result<HashMap<String, String>, ServoStyleError> {
        let combined_css = self.stylesheets.join("\n");
        
        let query = StyleQuery {
            id: uuid::Uuid::new_v4().to_string(),
            html: self.base_html.clone(),
            css: combined_css,
            selector: selector.to_string(),
            property: None, // Request all properties
        };

        let response = self.query_servo_process(query).await?;
        
        if response.success {
            response.computed_styles.ok_or_else(|| {
                ServoStyleError::ComputationError("No computed styles returned".to_string())
            })
        } else {
            Err(ServoStyleError::ComputationError(
                response.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }
}

/// Convenience function for computing a single CSS property using real Servo-Stylo integration
pub async fn compute_style_with_servo_real(
    html: &str,
    css: &str,
    selector: &str,
    property: &str,
    servo_path: Option<String>,
) -> Result<String, ServoStyleError> {
    let mut engine = ServoStyleEngineReal::with_servo_path(servo_path)?;
    engine.set_html(html)?;
    engine.add_stylesheet(css)?;
    engine.get_computed_style(selector, property).await
}
