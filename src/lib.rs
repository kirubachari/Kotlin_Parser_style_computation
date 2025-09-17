//! # Stylo Compute - CSS Style Engine via Servo
//!
//! A Rust implementation that computes CSS styles for DOM elements using Servo as an
//! intermediary to access Stylo's native APIs. This approach leverages Servo's existing
//! `getComputedStyle()` implementation which directly uses Stylo's style resolution pipeline.
//!
//! ## Overview
//!
//! This library provides methods to compute CSS styles equivalent to the standard
//! `getComputedStyle()` web API by using Servo's browser engine, which has complete
//! integration with Stylo's CSS engine including:
//!
//! - `resolve_style()` - Stylo's core style resolution function
//! - `SharedStyleContext` - Stylo's style computation context
//! - `ComputedValues` - Stylo's computed property values
//! - Complete CSS cascade and inheritance handling
//! - Media query evaluation and responsive design support
//! - Animation and transition support
//! - Pseudo-element handling
//!
//! ## Architecture
//!
//! ```text
//! Your Application
//!       ↓
//! ServoStyleEngine (this library)
//!       ↓
//! Servo Browser Engine
//!       ↓
//! Stylo CSS Engine (native APIs)
//!       ↓
//! Computed CSS Values
//! ```
//!
//! ## Example
//!
//! ```rust
//! use stylo_compute::ServoStyleEngine;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a Servo-based style engine
//!     let mut engine = ServoStyleEngine::new()?;
//!
//!     // Set HTML content
//!     engine.set_html(r#"
//!         <div class="highlight" id="main">
//!             <p>Hello, World!</p>
//!         </div>
//!     "#)?;
//!
//!     // Add CSS stylesheets
//!     engine.add_stylesheet(r#"
//!         .highlight {
//!             color: red;
//!             font-size: 24px;
//!             background-color: yellow;
//!             font-weight: bold;
//!         }
//!         p {
//!             margin: 10px;
//!             padding: 5px;
//!         }
//!     "#)?;
//!
//!     // Get computed style for a specific property (uses Servo's getComputedStyle)
//!     let color = engine.get_computed_style(".highlight", "color", None).await?;
//!     println!("Computed color: {}", color); // "rgb(255, 0, 0)"
//!
//!     // Get all computed styles for an element
//!     let all_styles = engine.get_all_computed_styles(".highlight", None).await?;
//!     for (property, value) in all_styles {
//!         println!("{}: {}", property, value);
//!     }
//!
//!     Ok(())
//! }
//! ```

mod servo_style_engine_real;
mod servo_style_engine_optimized;

pub use servo_style_engine_real::{ServoStyleEngineReal, ServoStyleError, compute_style_with_servo_real};
pub use servo_style_engine_optimized::{ServoStyleEngineOptimized, compute_styles_batch_optimized};







