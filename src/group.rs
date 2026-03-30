//! Group dynamics — cohesion, social loafing, groupthink, Tuckman stages.

use serde::{Deserialize, Serialize};

use crate::error::{Result, validate_finite};

/// Tuckman's stages of group development.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TuckmanStage {
    /// Group is forming, members are cautious.
    Forming,
    /// Conflict and competition emerge.
    Storming,
    /// Group establishes norms and cohesion.
    Norming,
    /// Group is functioning effectively.
    Performing,
    /// Group disbands or transitions.
    Adjourning,
}

/// Social loafing: Ringelmann effect — individual effort decreases with group size.
///
/// `effort_per_person = individual_effort * (1 - loss_factor * ln(group_size))`
///
/// The loss factor typically ranges from 0.05 to 0.15.
///
/// # Errors
///
/// Returns error if parameters are invalid.
#[inline]
#[must_use = "returns the effort per person without side effects"]
pub fn social_loafing(group_size: usize, individual_effort: f64, loss_factor: f64) -> Result<f64> {
    validate_finite(individual_effort, "individual_effort")?;
    validate_finite(loss_factor, "loss_factor")?;
    if group_size == 0 {
        return Ok(0.0);
    }
    let reduction = 1.0 - loss_factor * (group_size as f64).ln();
    Ok(individual_effort * reduction.max(0.1)) // minimum 10% effort
}

/// Groupthink risk assessment.
///
/// Returns a risk score between 0.0 (no risk) and 1.0 (high risk).
///
/// Based on Janis's antecedent conditions: high cohesion, insulation
/// from external opinions, and directive leadership.
///
/// # Errors
///
/// Returns error if parameters are non-finite.
#[must_use = "returns the groupthink risk without side effects"]
pub fn groupthink_risk(cohesion: f64, insulation: f64, leader_bias: f64) -> Result<f64> {
    validate_finite(cohesion, "cohesion")?;
    validate_finite(insulation, "insulation")?;
    validate_finite(leader_bias, "leader_bias")?;
    // All inputs should be 0-1
    let c = cohesion.clamp(0.0, 1.0);
    let ins = insulation.clamp(0.0, 1.0);
    let lb = leader_bias.clamp(0.0, 1.0);
    // Weighted combination
    Ok((0.4 * c + 0.3 * ins + 0.3 * lb).clamp(0.0, 1.0))
}

/// Collective intelligence factor.
///
/// Based on Woolley et al. (2010): diversity, independence of thought,
/// and decentralization of opinion contribute to group intelligence.
///
/// Returns a score from 0.0 to 1.0.
///
/// # Errors
///
/// Returns error if parameters are non-finite.
#[must_use = "returns the collective intelligence factor without side effects"]
pub fn collective_intelligence(
    diversity: f64,
    independence: f64,
    decentralization: f64,
) -> Result<f64> {
    validate_finite(diversity, "diversity")?;
    validate_finite(independence, "independence")?;
    validate_finite(decentralization, "decentralization")?;
    let d = diversity.clamp(0.0, 1.0);
    let i = independence.clamp(0.0, 1.0);
    let dec = decentralization.clamp(0.0, 1.0);
    Ok(((d + i + dec) / 3.0).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_loafing_individual() {
        let effort = social_loafing(1, 100.0, 0.1).unwrap();
        assert!((effort - 100.0).abs() < 1e-10); // ln(1) = 0
    }

    #[test]
    fn test_social_loafing_group() {
        let effort = social_loafing(10, 100.0, 0.1).unwrap();
        assert!(effort < 100.0); // decreased effort
    }

    #[test]
    fn test_groupthink_low_risk() {
        let risk = groupthink_risk(0.2, 0.1, 0.1).unwrap();
        assert!(risk < 0.2);
    }

    #[test]
    fn test_groupthink_high_risk() {
        let risk = groupthink_risk(0.9, 0.9, 0.9).unwrap();
        assert!(risk > 0.8);
    }

    #[test]
    fn test_collective_intelligence() {
        let ci = collective_intelligence(0.8, 0.7, 0.9).unwrap();
        assert!(ci > 0.7);
    }

    #[test]
    fn test_tuckman_ordering() {
        assert!(TuckmanStage::Forming < TuckmanStage::Storming);
        assert!(TuckmanStage::Storming < TuckmanStage::Norming);
        assert!(TuckmanStage::Norming < TuckmanStage::Performing);
    }

    #[test]
    fn test_tuckman_serde_roundtrip() {
        let stage = TuckmanStage::Performing;
        let json = serde_json::to_string(&stage).unwrap();
        let back: TuckmanStage = serde_json::from_str(&json).unwrap();
        assert_eq!(stage, back);
    }

    #[test]
    fn test_social_loafing_zero_group() {
        let effort = social_loafing(0, 100.0, 0.1).unwrap();
        assert!((effort - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_social_loafing_minimum_floor() {
        // Very large group should hit the 10% floor
        let effort = social_loafing(1_000_000, 100.0, 0.5).unwrap();
        assert!((effort - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_groupthink_risk_clamping() {
        // Out-of-range inputs should be clamped
        let risk = groupthink_risk(1.5, -0.5, 2.0).unwrap();
        assert!((0.0..=1.0).contains(&risk));
    }

    #[test]
    fn test_groupthink_risk_boundaries() {
        let zero = groupthink_risk(0.0, 0.0, 0.0).unwrap();
        assert!(zero.abs() < 1e-10);
        let one = groupthink_risk(1.0, 1.0, 1.0).unwrap();
        assert!((one - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_collective_intelligence_boundaries() {
        let zero = collective_intelligence(0.0, 0.0, 0.0).unwrap();
        assert!(zero.abs() < 1e-10);
        let one = collective_intelligence(1.0, 1.0, 1.0).unwrap();
        assert!((one - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_social_loafing_nan_error() {
        assert!(social_loafing(5, f64::NAN, 0.1).is_err());
    }
}
