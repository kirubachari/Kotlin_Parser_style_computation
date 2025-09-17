use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::process::Stdio;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tempfile::NamedTempFile;
use tokio::sync::OnceCell;
use tokio::process::{Child, Command};

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
    #[error("Servo daemon not available: {0}")]
    DaemonError(String),
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
struct BatchQuery {
    queries: Vec<StyleQuery>,
}

#[derive(Serialize, Deserialize, Debug)]
struct StyleResponse {
    id: String,
    success: bool,
    computed_value: Option<String>,
    computed_styles: Option<HashMap<String, String>>,
    error: Option<String>,
}

/// Servo daemon process manager
#[allow(dead_code)]
struct ServoDaemon {
    process: Child,
    servo_path: String,
}

impl ServoDaemon {
    async fn start(servo_path: String) -> Result<Self, ServoStyleError> {
        println!("🚀 Starting Servo daemon for persistent style computation...");
        
        // Start Servo in daemon-like mode with a simple server page
        let daemon_html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Servo Style Daemon</title>
    <style>
        body { font-family: monospace; padding: 20px; }
        .status { color: green; }
    </style>
</head>
<body>
    <h1 class="status">Servo Style Computation Daemon</h1>
    <p>Ready for style queries...</p>
    <script>
        console.log('SERVO_DAEMON_READY');
        // Keep alive with periodic heartbeat
        setInterval(() => {
            console.log('SERVO_DAEMON_HEARTBEAT:' + Date.now());
        }, 5000);
    </script>
</body>
</html>"#;

        // Create daemon HTML file
        let daemon_file = "/tmp/servo_daemon.html";
        std::fs::write(daemon_file, daemon_html)?;

        let process = Command::new(&servo_path)
            .arg("--headless")
            .arg(format!("file://{}", daemon_file))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        println!("✅ Servo daemon started");

        Ok(ServoDaemon {
            process,
            servo_path,
        })
    }

    #[allow(dead_code)]
    async fn is_alive(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(Some(_)) => false, // Process has exited
            Ok(None) => true,     // Process is still running
            Err(_) => false,      // Error checking process
        }
    }

    #[allow(dead_code)]
    async fn restart(&mut self) -> Result<(), ServoStyleError> {
        println!("🔄 Restarting Servo daemon...");
        let _ = self.process.kill().await;
        let new_daemon = ServoDaemon::start(self.servo_path.clone()).await?;
        self.process = new_daemon.process;
        Ok(())
    }
}

/// Global daemon instance
static SERVO_DAEMON: OnceCell<Arc<Mutex<ServoDaemon>>> = OnceCell::const_new();

/// Optimized Servo-based CSS style engine with daemon mode and batch processing
pub struct ServoStyleEngineOptimized {
    base_html: String,
    stylesheets: Vec<String>,
    servo_path: Option<String>,
    use_daemon: bool,
    #[allow(dead_code)]
    batch_size: usize,
}

impl ServoStyleEngineOptimized {
    /// Create a new optimized ServoStyleEngine instance
    pub fn new() -> Result<Self, ServoStyleError> {
        Self::with_options(None, true, 5)
    }

    /// Create with custom options
    pub fn with_options(
        servo_path: Option<String>, 
        use_daemon: bool, 
        batch_size: usize
    ) -> Result<Self, ServoStyleError> {
        // Check if Servo is available
        if let Some(ref path) = servo_path {
            if !std::path::Path::new(path).exists() {
                return Err(ServoStyleError::ServoNotFound);
            }
        } else if which::which("servo").is_err() {
            return Err(ServoStyleError::ServoNotFound);
        }
        
        println!("✅ Servo found - enabling optimized Stylo integration");
        if use_daemon {
            println!("   🔧 Daemon mode enabled for persistent computation");
        }
        println!("   📦 Batch size: {} queries", batch_size);
        
        if let Some(ref path) = servo_path {
            println!("   Using custom Servo path: {}", path);
        } else {
            println!("   Using Servo from PATH");
        }

        Ok(ServoStyleEngineOptimized {
            base_html: String::new(),
            stylesheets: Vec::new(),
            servo_path,
            use_daemon,
            batch_size,
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

    /// Initialize daemon if needed
    async fn ensure_daemon(&self) -> Result<(), ServoStyleError> {
        if !self.use_daemon {
            return Ok(());
        }

        let servo_path = self.servo_path.as_deref().unwrap_or("servo").to_string();
        
        SERVO_DAEMON.get_or_try_init(|| async {
            let daemon = ServoDaemon::start(servo_path).await?;
            Ok(Arc::new(Mutex::new(daemon)))
        }).await.map_err(|e: ServoStyleError| e)?;

        Ok(())
    }

    /// Create an HTML file with embedded JavaScript for batch queries
    fn create_batch_html(&self, queries: &[StyleQuery]) -> String {
        let combined_css = self.stylesheets.join("\n");
        
        // Generate JavaScript for all queries
        let mut js_queries = String::new();
        for query in queries {
            let query_js = if let Some(ref prop) = query.property {
                format!(r#"
                    try {{
                        var element = document.querySelector('{}');
                        if (element) {{
                            var computedStyle = window.getComputedStyle(element);
                            var value = computedStyle.getPropertyValue('{}');
                            console.log('COMPUTED_STYLE_RESULT:{}:' + JSON.stringify({{
                                id: '{}',
                                selector: '{}',
                                property: '{}',
                                value: value
                            }}));
                        }} else {{
                            console.log('COMPUTED_STYLE_ERROR:{}:Element not found');
                        }}
                    }} catch (e) {{
                        console.log('COMPUTED_STYLE_ERROR:{}:' + e.message);
                    }}
                "#, query.selector, prop, query.id, query.id, query.selector, prop, query.id, query.id)
            } else {
                format!(r#"
                    try {{
                        var element = document.querySelector('{}');
                        if (element) {{
                            var computedStyle = window.getComputedStyle(element);
                            var styles = {{}};
                            for (var i = 0; i < computedStyle.length; i++) {{
                                var propName = computedStyle[i];
                                styles[propName] = computedStyle.getPropertyValue(propName);
                            }}
                            console.log('COMPUTED_STYLES_RESULT:{}:' + JSON.stringify({{
                                id: '{}',
                                selector: '{}',
                                styles: styles
                            }}));
                        }} else {{
                            console.log('COMPUTED_STYLE_ERROR:{}:Element not found');
                        }}
                    }} catch (e) {{
                        console.log('COMPUTED_STYLE_ERROR:{}:' + e.message);
                    }}
                "#, query.selector, query.id, query.id, query.selector, query.id, query.id)
            };
            js_queries.push_str(&query_js);
        }

        let script = format!(r#"
            window.addEventListener('load', function() {{
                console.log('BATCH_START:{}');
                {}
                console.log('BATCH_END:{}');
                setTimeout(function() {{ window.close(); }}, 200);
            }});
        "#, queries.len(), js_queries, queries.len());

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

    /// Process queries in batch using optimized Servo
    async fn process_batch(&self, queries: Vec<StyleQuery>) -> Result<Vec<StyleResponse>, ServoStyleError> {
        if self.use_daemon {
            self.process_batch_with_daemon(queries).await
        } else {
            self.process_batch_standalone(queries).await
        }
    }

    /// Process batch with daemon (reuses Servo instance)
    async fn process_batch_with_daemon(&self, queries: Vec<StyleQuery>) -> Result<Vec<StyleResponse>, ServoStyleError> {
        self.ensure_daemon().await?;
        
        // For now, fall back to standalone mode as daemon communication needs more work
        // TODO: Implement proper daemon communication protocol
        println!("🔄 Daemon mode: falling back to optimized standalone for now");
        self.process_batch_standalone(queries).await
    }

    /// Process batch with standalone Servo (one instance per batch)
    async fn process_batch_standalone(&self, queries: Vec<StyleQuery>) -> Result<Vec<StyleResponse>, ServoStyleError> {
        let html_content = self.create_batch_html(&queries);
        let servo_cmd = self.servo_path.as_deref().unwrap_or("servo");
        
        // Create temp file
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| ServoStyleError::CommunicationError(format!("Failed to create temp file: {}", e)))?;
        temp_file.write_all(html_content.as_bytes())
            .map_err(|e| ServoStyleError::CommunicationError(format!("Failed to write temp file: {}", e)))?;
        let temp_path = temp_file.path();

        // Save debug file
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let debug_path = format!("/tmp/debug_servo_batch_{}.html", timestamp);
        std::fs::write(&debug_path, &html_content)?;
        
        println!("🚀 Processing batch of {} queries...", queries.len());
        println!("   Debug file: {}", debug_path);

        // Run Servo with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            Command::new(servo_cmd)
                .arg("--headless")
                .arg(format!("file://{}", temp_path.display()))
                .output()
        ).await;

        match output {
            Ok(Ok(process_output)) => {
                let stdout = String::from_utf8_lossy(&process_output.stdout);
                let stderr = String::from_utf8_lossy(&process_output.stderr);
                
                // Save batch results
                let result_file = format!("/tmp/servo_batch_results_{}.txt", timestamp);
                let content = format!("BATCH RESULTS\n=============\n{} queries processed\n\nSTDOUT:\n{}\n\nSTDERR:\n{}\n", 
                    queries.len(), stdout, stderr);
                std::fs::write(&result_file, content)?;
                println!("   📄 Batch results saved to: {}", result_file);
                
                self.parse_batch_output(&stdout, &stderr, &queries)
            },
            Ok(Err(e)) => Err(ServoStyleError::CommunicationError(format!("Failed to start Servo: {}", e))),
            Err(_) => Err(ServoStyleError::CommunicationError("Servo batch timed out".to_string())),
        }
    }

    /// Parse batch output and match results to queries
    fn parse_batch_output(&self, stdout: &str, stderr: &str, queries: &[StyleQuery]) -> Result<Vec<StyleResponse>, ServoStyleError> {
        let mut responses = Vec::new();
        let mut processed_ids = std::collections::HashSet::new();

        for line in stdout.lines().chain(stderr.lines()) {
            // Parse single property results
            if line.contains("COMPUTED_STYLE_RESULT:") {
                if let Some(parts) = line.split("COMPUTED_STYLE_RESULT:").nth(1) {
                    if let Some((id, json_part)) = parts.split_once(':') {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_part) {
                            if let Some(value) = parsed["value"].as_str() {
                                responses.push(StyleResponse {
                                    id: id.to_string(),
                                    success: true,
                                    computed_value: Some(value.to_string()),
                                    computed_styles: None,
                                    error: None,
                                });
                                processed_ids.insert(id.to_string());
                                println!("   ✅ Batch result for {}: {}", id, value);
                            }
                        }
                    }
                }
            }
            
            // Parse all styles results
            if line.contains("COMPUTED_STYLES_RESULT:") {
                if let Some(parts) = line.split("COMPUTED_STYLES_RESULT:").nth(1) {
                    if let Some((id, json_part)) = parts.split_once(':') {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_part) {
                            if let Some(styles) = parsed["styles"].as_object() {
                                let styles_map: HashMap<String, String> = styles.iter()
                                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                                    .collect();
                                responses.push(StyleResponse {
                                    id: id.to_string(),
                                    success: true,
                                    computed_value: None,
                                    computed_styles: Some(styles_map),
                                    error: None,
                                });
                                processed_ids.insert(id.to_string());
                                println!("   ✅ Batch styles for {}: {} properties", id, styles.len());
                            }
                        }
                    }
                }
            }
        }

        // Add error responses for unprocessed queries
        for query in queries {
            if !processed_ids.contains(&query.id) {
                responses.push(StyleResponse {
                    id: query.id.clone(),
                    success: false,
                    computed_value: None,
                    computed_styles: None,
                    error: Some("No result found in batch output".to_string()),
                });
            }
        }

        Ok(responses)
    }

    /// Get computed style for a specific CSS property (optimized)
    pub async fn get_computed_style(&mut self, selector: &str, property: &str) -> Result<String, ServoStyleError> {
        let query = StyleQuery {
            id: uuid::Uuid::new_v4().to_string(),
            html: self.base_html.clone(),
            css: self.stylesheets.join("\n"),
            selector: selector.to_string(),
            property: Some(property.to_string()),
        };

        let responses = self.process_batch(vec![query]).await?;
        
        if let Some(response) = responses.into_iter().next() {
            if response.success {
                response.computed_value.ok_or_else(|| {
                    ServoStyleError::ComputationError("No computed value returned".to_string())
                })
            } else {
                Err(ServoStyleError::ComputationError(
                    response.error.unwrap_or_else(|| "Unknown error".to_string())
                ))
            }
        } else {
            Err(ServoStyleError::ComputationError("No response received".to_string()))
        }
    }

    /// Get all computed styles for an element (optimized)
    pub async fn get_all_computed_styles(&mut self, selector: &str) -> Result<HashMap<String, String>, ServoStyleError> {
        let query = StyleQuery {
            id: uuid::Uuid::new_v4().to_string(),
            html: self.base_html.clone(),
            css: self.stylesheets.join("\n"),
            selector: selector.to_string(),
            property: None,
        };

        let responses = self.process_batch(vec![query]).await?;
        
        if let Some(response) = responses.into_iter().next() {
            if response.success {
                response.computed_styles.ok_or_else(|| {
                    ServoStyleError::ComputationError("No computed styles returned".to_string())
                })
            } else {
                Err(ServoStyleError::ComputationError(
                    response.error.unwrap_or_else(|| "Unknown error".to_string())
                ))
            }
        } else {
            Err(ServoStyleError::ComputationError("No response received".to_string()))
        }
    }

    /// Process multiple style queries efficiently in batch
    pub async fn compute_styles_batch(&mut self, requests: Vec<(String, Option<String>)>) -> Result<Vec<(String, Result<String, ServoStyleError>)>, ServoStyleError> {
        let mut queries = Vec::new();
        
        // Clone requests for later use
        let requests_clone = requests.clone();
        
        for (selector, property) in requests {
            queries.push(StyleQuery {
                id: uuid::Uuid::new_v4().to_string(),
                html: self.base_html.clone(),
                css: self.stylesheets.join("\n"),
                selector: selector.clone(),
                property: property.clone(),
            });
        }

        let responses = self.process_batch(queries).await?;
        
        let mut results = Vec::new();
        for (i, response) in responses.into_iter().enumerate() {
            let selector = if i < requests_clone.len() { 
                requests_clone[i].0.clone() 
            } else { 
                format!("query_{}", i) 
            };
            
            let result = if response.success {
                if let Some(value) = response.computed_value {
                    Ok(value)
                } else if let Some(styles) = response.computed_styles {
                    Ok(serde_json::to_string(&styles).unwrap_or_default())
                } else {
                    Err(ServoStyleError::ComputationError("No result data".to_string()))
                }
            } else {
                Err(ServoStyleError::ComputationError(
                    response.error.unwrap_or_else(|| "Unknown error".to_string())
                ))
            };
            
            results.push((selector, result));
        }

        Ok(results)
    }
}

/// Convenience function for optimized batch style computation
pub async fn compute_styles_batch_optimized(
    html: &str,
    css: &str,
    queries: Vec<(String, String, Option<String>)>, // (selector, property_name, property_value)
    servo_path: Option<String>,
) -> Result<Vec<(String, Result<String, ServoStyleError>)>, ServoStyleError> {
    let mut engine = ServoStyleEngineOptimized::with_options(servo_path, true, 10)?;
    engine.set_html(html)?;
    engine.add_stylesheet(css)?;
    
    let requests: Vec<(String, Option<String>)> = queries.into_iter()
        .map(|(selector, prop, _)| (selector, Some(prop)))
        .collect();
    
    engine.compute_styles_batch(requests).await
}