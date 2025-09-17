use thiserror::Error;

/// Errors that can occur during style computation
#[derive(Error, Debug)]
pub enum StyleError {
    #[error("Invalid DOM element: {0}")]
    InvalidElement(String),
    
    #[error("Style computation failed: {0}")]
    ComputationFailed(String),
    
    #[error("CSS parsing error: {0}")]
    CssParsingError(String),
    
    #[error("Missing required stylesheet")]
    MissingStylesheet,
    
    #[error("Invalid CSS property: {0}")]
    InvalidProperty(String),
    
    #[error("Style context not initialized")]
    ContextNotInitialized,
    
    #[error("DOM traversal error: {0}")]
    TraversalError(String),
    
    #[error("Internal Stylo error: {0}")]
    StyloError(String),
}

/// Result type for style operations
pub type StyleResult<T> = Result<T, StyleError>;

// CSS parsing error conversion would be implemented here when using actual CSS parser
