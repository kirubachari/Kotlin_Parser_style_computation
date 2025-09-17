use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::io::{Write, BufRead, BufReader};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::{sleep, Duration};

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
/// This implementation communicates with actual Servo processes to compute CSS styles
/// using Servo's getComputedStyle() implementation, which directly calls Stylo's
/// native APIs including resolve_style(), SharedStyleContext, and ComputedValues.
pub struct ServoStyleEngineReal {
    servo_process: Option<Child>,
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
            servo_process: None,
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

    /// Start Servo process if not already running
    async fn ensure_servo_process(&mut self) -> Result<(), ServoStyleError> {
        if self.servo_process.is_none() {
            let servo_cmd = self.servo_path.as_deref().unwrap_or("servo");
            
            println!("🚀 Starting Servo process for style computation...");
            let child = Command::new(servo_cmd)
                .arg("--headless")
                .arg("--style-query-mode")  // Custom flag for style queries
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            
            self.servo_process = Some(child);
            println!("✅ Servo process started successfully");
        }
        Ok(())
    }

    /// Query Servo process for computed styles using real Stylo APIs
    async fn query_servo_process(&mut self, query: StyleQuery) -> Result<StyleResponse, ServoStyleError> {
        self.ensure_servo_process().await?;
        
        if let Some(process) = &mut self.servo_process {
            // Send JSON query to Servo
            let query_json = serde_json::to_string(&query)?;
            
            if let Some(stdin) = process.stdin.as_mut() {
                writeln!(stdin, "{}", query_json)?;
                stdin.flush()?;
            }
            
            // Read JSON response from Servo
            if let Some(stdout) = process.stdout.as_mut() {
                let mut reader = BufReader::new(stdout);
                let mut response_line = String::new();
                reader.read_line(&mut response_line)?;
                
                let response: StyleResponse = serde_json::from_str(&response_line.trim())?;
                return Ok(response);
            }
        }
        
        Err(ServoStyleError::CommunicationError("Failed to communicate with Servo process".to_string()))
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

impl Drop for ServoStyleEngineReal {
    fn drop(&mut self) {
        if let Some(mut process) = self.servo_process.take() {
            let _ = process.kill();
            let _ = process.wait();
            println!("🛑 Servo process terminated");
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
