//! Error types for the sangha sociology engine.

use serde::{Deserialize, Serialize};

/// Errors that can occur in sangha operations.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[non_exhaustive]
pub enum SanghaError {
    /// A network structure was invalid.
    #[error("invalid network: {0}")]
    InvalidNetwork(String),

    /// A population parameter was invalid.
    #[error("invalid population: {0}")]
    InvalidPopulation(String),

    /// A simulation failed to converge or complete.
    #[error("simulation failed: {0}")]
    SimulationFailed(String),

    /// A general computation error.
    #[error("computation error: {0}")]
    ComputationError(String),
}

/// Result type alias for sangha operations.
pub type Result<T> = core::result::Result<T, SanghaError>;

/// Validate that a value is finite and non-NaN.
#[inline]
pub(crate) fn validate_finite(value: f64, name: &str) -> Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SanghaError::ComputationError(format!(
            "{name} must be finite, got {value}"
        )))
    }
}

/// Validate that a value is positive (> 0).
#[inline]
pub(crate) fn validate_positive(value: f64, name: &str) -> Result<()> {
    validate_finite(value, name)?;
    if value > 0.0 {
        Ok(())
    } else {
        Err(SanghaError::ComputationError(format!(
            "{name} must be positive, got {value}"
        )))
    }
}

/// Validate that a value is non-negative (>= 0).
#[inline]
pub(crate) fn validate_non_negative(value: f64, name: &str) -> Result<()> {
    validate_finite(value, name)?;
    if value >= 0.0 {
        Ok(())
    } else {
        Err(SanghaError::ComputationError(format!(
            "{name} must be non-negative, got {value}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SanghaError::InvalidNetwork("test".into());
        assert_eq!(err.to_string(), "invalid network: test");
    }

    #[test]
    fn test_error_serde_roundtrip() {
        let err = SanghaError::SimulationFailed("diverged".into());
        let json = serde_json::to_string(&err).unwrap();
        let back: SanghaError = serde_json::from_str(&json).unwrap();
        assert_eq!(err.to_string(), back.to_string());
    }

    #[test]
    fn test_validate_finite() {
        assert!(validate_finite(1.0, "x").is_ok());
        assert!(validate_finite(f64::NAN, "x").is_err());
    }

    #[test]
    fn test_validate_positive() {
        assert!(validate_positive(1.0, "x").is_ok());
        assert!(validate_positive(0.0, "x").is_err());
        assert!(validate_positive(-1.0, "x").is_err());
    }
}
