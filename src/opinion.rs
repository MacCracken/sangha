//! Opinion dynamics — voter model, bounded confidence, polarization.

use serde::{Deserialize, Serialize};

use crate::error::{Result, SanghaError, validate_finite, validate_positive};

/// Opinion state: continuous value between 0.0 and 1.0.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Opinion(pub f64);

impl Opinion {
    /// Create a new opinion value, clamped to [0, 1].
    #[inline]
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
}

/// Deffuant bounded confidence model step.
///
/// Two agents interact and move toward each other's opinion only if
/// their opinions differ by less than `threshold`. The convergence
/// parameter `mu` controls the step size (0.0 = no change, 0.5 = meet
/// in the middle).
///
/// Returns `(new_opinion1, new_opinion2)`.
///
/// # Errors
///
/// Returns error if parameters are invalid.
#[must_use = "returns the updated opinions without side effects"]
pub fn deffuant_interaction(
    opinion1: f64,
    opinion2: f64,
    threshold: f64,
    mu: f64,
) -> Result<(f64, f64)> {
    validate_finite(opinion1, "opinion1")?;
    validate_finite(opinion2, "opinion2")?;
    validate_finite(threshold, "threshold")?;
    validate_finite(mu, "mu")?;

    let diff = (opinion1 - opinion2).abs();
    if diff < threshold {
        let new1 = opinion1 + mu * (opinion2 - opinion1);
        let new2 = opinion2 + mu * (opinion1 - opinion2);
        Ok((new1, new2))
    } else {
        Ok((opinion1, opinion2))
    }
}

/// Echo chamber index: how much a network's opinion clusters differ.
///
/// Returns a value between 0 (no echo chambers) and 1 (complete polarization).
///
/// Computed as the variance of opinions normalized by maximum possible variance.
///
/// # Errors
///
/// Returns error if opinions is empty.
#[must_use = "returns the echo chamber index without side effects"]
pub fn echo_chamber_index(opinions: &[f64]) -> Result<f64> {
    if opinions.is_empty() {
        return Err(SanghaError::ComputationError(
            "need at least one opinion".into(),
        ));
    }

    let n = opinions.len() as f64;
    let mean = opinions.iter().sum::<f64>() / n;
    let variance = opinions.iter().map(|&o| (o - mean).powi(2)).sum::<f64>() / n;

    // Maximum variance for [0,1] opinions is 0.25 (all at 0 or 1)
    Ok((variance / 0.25).min(1.0))
}

/// Check if opinions have reached consensus (all within `epsilon` of mean).
///
/// # Errors
///
/// Returns error if `epsilon` is non-positive or non-finite.
#[must_use = "returns whether consensus has been reached without side effects"]
pub fn has_consensus(opinions: &[f64], epsilon: f64) -> Result<bool> {
    validate_positive(epsilon, "epsilon")?;
    if opinions.is_empty() {
        return Ok(true);
    }
    let n = opinions.len() as f64;
    let mean = opinions.iter().sum::<f64>() / n;
    Ok(opinions.iter().all(|&o| (o - mean).abs() < epsilon))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deffuant_within_threshold() {
        let (o1, o2) = deffuant_interaction(0.3, 0.5, 0.5, 0.5).unwrap();
        // Should converge toward each other
        assert!((o1 - 0.4).abs() < 1e-10);
        assert!((o2 - 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_deffuant_outside_threshold() {
        let (o1, o2) = deffuant_interaction(0.1, 0.9, 0.3, 0.5).unwrap();
        // No change when beyond threshold
        assert!((o1 - 0.1).abs() < 1e-10);
        assert!((o2 - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_echo_chamber_consensus() {
        let opinions = vec![0.5, 0.5, 0.5, 0.5];
        let idx = echo_chamber_index(&opinions).unwrap();
        assert!(idx.abs() < 1e-10);
    }

    #[test]
    fn test_echo_chamber_polarized() {
        let opinions = vec![0.0, 0.0, 1.0, 1.0];
        let idx = echo_chamber_index(&opinions).unwrap();
        assert!(idx > 0.9);
    }

    #[test]
    fn test_has_consensus_true() {
        assert!(has_consensus(&[0.5, 0.5, 0.5], 0.01).unwrap());
    }

    #[test]
    fn test_has_consensus_false() {
        assert!(!has_consensus(&[0.1, 0.9], 0.01).unwrap());
    }

    #[test]
    fn test_has_consensus_invalid_epsilon() {
        assert!(has_consensus(&[0.5], -1.0).is_err());
        assert!(has_consensus(&[0.5], 0.0).is_err());
        assert!(has_consensus(&[0.5], f64::NAN).is_err());
    }

    #[test]
    fn test_opinion_serde_roundtrip() {
        let o = Opinion::new(0.7);
        let json = serde_json::to_string(&o).unwrap();
        let back: Opinion = serde_json::from_str(&json).unwrap();
        assert!((o.0 - back.0).abs() < 1e-10);
    }
}
