use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
}

impl ConversionError {
    pub fn unsupported(from: &str, to: &str) -> Self {
        Self {
            code: "UNSUPPORTED_CONVERSION".to_string(),
            message: format!("Conversion from {} to {} is not supported", from, to),
            details: None,
            recoverable: false,
        }
    }

    pub fn corrupt_input(details: &str) -> Self {
        Self {
            code: "CORRUPT_INPUT".to_string(),
            message: "The input file appears to be corrupted".to_string(),
            details: Some(details.to_string()),
            recoverable: false,
        }
    }

    pub fn engine_failure(engine: &str, details: &str) -> Self {
        Self {
            code: "ENGINE_FAILURE".to_string(),
            message: format!("Conversion engine '{}' encountered an error", engine),
            details: Some(details.to_string()),
            recoverable: true,
        }
    }

    pub fn missing_engine(engine: &str) -> Self {
        Self {
            code: "MISSING_ENGINE".to_string(),
            message: format!("Required conversion engine is not available: {}", engine),
            details: None,
            recoverable: false,
        }
    }

    pub fn timeout() -> Self {
        Self {
            code: "TIMEOUT".to_string(),
            message: "Conversion timed out".to_string(),
            details: None,
            recoverable: true,
        }
    }

    pub fn password_protected() -> Self {
        Self {
            code: "PASSWORD_PROTECTED".to_string(),
            message: "This file is password-protected".to_string(),
            details: None,
            recoverable: false,
        }
    }

    pub fn io_error(msg: &str) -> Self {
        Self {
            code: "IO_ERROR".to_string(),
            message: msg.to_string(),
            details: None,
            recoverable: false,
        }
    }

    pub fn validation_failed(msg: &str) -> Self {
        Self {
            code: "VALIDATION_FAILED".to_string(),
            message: msg.to_string(),
            details: None,
            recoverable: false,
        }
    }
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ConversionError {}
