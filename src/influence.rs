//! Social influence — conformity, diffusion, social proof.

use crate::error::{Result, validate_finite};

/// Asch-inspired conformity model.
///
/// Returns `true` if the individual conforms to group pressure.
/// Conformity increases with group size and pressure, decreases with conviction.
///
/// # Errors
///
/// Returns error if parameters are non-finite.
#[must_use = "returns the conformity decision without side effects"]
pub fn conformity_threshold(
    individual_conviction: f64,
    group_pressure: f64,
    group_size: usize,
) -> Result<bool> {
    validate_finite(individual_conviction, "individual_conviction")?;
    validate_finite(group_pressure, "group_pressure")?;
    // Size effect: conformity peaks around 3-4 unanimously opposing members
    let size_effect = 1.0 - (-0.5 * group_size as f64).exp();
    let total_pressure = group_pressure * size_effect;
    Ok(total_pressure > individual_conviction)
}

/// Social proof weight: fraction of population that has adopted.
///
/// As more people adopt, social proof increases, making adoption more likely.
///
/// # Errors
///
/// Returns error if population is zero.
#[inline]
#[must_use = "returns the social proof weight without side effects"]
pub fn social_proof_weight(adopters: usize, population: usize) -> Result<f64> {
    if population == 0 {
        return Err(crate::error::SanghaError::InvalidPopulation(
            "population must be > 0".into(),
        ));
    }
    Ok(adopters as f64 / population as f64)
}

/// Bass diffusion model: rate of new adoption.
///
/// `f(t) = (p + q * F(t)) * (1 - F(t))`
///
/// where `p` is the coefficient of innovation (external influence),
/// `q` is the coefficient of imitation (internal influence), and
/// `F(t)` is the current adoption fraction.
///
/// # Errors
///
/// Returns error if parameters are invalid.
#[inline]
#[must_use = "returns the adoption rate without side effects"]
pub fn bass_diffusion(adopters: usize, population: usize, p: f64, q: f64) -> Result<f64> {
    validate_finite(p, "p")?;
    validate_finite(q, "q")?;
    if population == 0 {
        return Err(crate::error::SanghaError::InvalidPopulation(
            "population must be > 0".into(),
        ));
    }
    let f_t = adopters as f64 / population as f64;
    Ok((p + q * f_t) * (1.0 - f_t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conformity_low_pressure() {
        let conforms = conformity_threshold(0.9, 0.2, 3).unwrap();
        assert!(!conforms);
    }

    #[test]
    fn test_conformity_high_pressure() {
        let conforms = conformity_threshold(0.3, 0.8, 5).unwrap();
        assert!(conforms);
    }

    #[test]
    fn test_social_proof_weight() {
        let w = social_proof_weight(25, 100).unwrap();
        assert!((w - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_social_proof_zero_pop() {
        assert!(social_proof_weight(0, 0).is_err());
    }

    #[test]
    fn test_bass_diffusion_early() {
        // Early adoption driven by innovation coefficient
        let rate = bass_diffusion(0, 1000, 0.03, 0.38).unwrap();
        assert!((rate - 0.03).abs() < 1e-10); // just p at start
    }

    #[test]
    fn test_bass_diffusion_saturated() {
        // Fully adopted: rate should be 0
        let rate = bass_diffusion(1000, 1000, 0.03, 0.38).unwrap();
        assert!((rate - 0.0).abs() < 1e-10);
    }
}
