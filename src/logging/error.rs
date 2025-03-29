use anyhow::Error;
use std::fmt::Write;
use tracing::Level;
use std::error::Source;

/// Extracts the full chain of causes from an anyhow::Error
pub fn extract_error_chain(error: &Error) -> String {
    let mut result = String::new();
    let mut current: Option<&dyn Source> = Some(error);
    let mut index = 0;

    while let Some(err) = current {
        if index > 0 {
            let _ = writeln!(result, "Caused by:");
        }
        let _ = writeln!(result, "  {}", err);
        
        current = err.source();
        index += 1;
    }

    result
}

/// Log an error with its full chain of causes if debug level is enabled
pub fn log_error(error: &Error, message: &str) {
    // Always log the top-level error message
    if tracing::enabled!(Level::DEBUG) {
        // Include the full error chain only at DEBUG level
        let chain = extract_error_chain(error);
        tracing::error!(
            error.message = %error,
            error.chain = chain,
            "{}", message
        );
    } else {
        // At higher levels (INFO, WARN, ERROR), only include the top-level error
        tracing::error!(
            error.message = %error,
            "{}", message
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{anyhow, Context};

    #[test]
    fn test_extract_error_chain() {
        // Create a chain of errors
        let root_error = anyhow!("Root error");
        let middle_error = root_error.context("Middle context");
        let top_error = middle_error.context("Top context");

        let chain = extract_error_chain(&top_error);
        
        // Check that the chain contains all three errors
        assert!(chain.contains("Top context"));
        assert!(chain.contains("Middle context"));
        assert!(chain.contains("Root error"));
    }
}