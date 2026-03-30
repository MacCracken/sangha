//! Inequality — Gini coefficient, Lorenz curve, distribution metrics.

extern crate alloc;
use alloc::vec::Vec;

use crate::error::{SanghaError, Result};

/// Gini coefficient: measure of income/wealth inequality.
///
/// `G = (2 * sum(i * y_i)) / (n * sum(y_i)) - (n + 1) / n`
///
/// G = 0: perfect equality. G = 1: perfect inequality.
///
/// # Errors
///
/// Returns [`SanghaError::ComputationError`] if the slice is empty or all values are zero.
#[must_use = "returns the Gini coefficient without side effects"]
pub fn gini_coefficient(incomes: &[f64]) -> Result<f64> {
    if incomes.is_empty() {
        return Err(SanghaError::ComputationError(
            "need at least one income value".into(),
        ));
    }

    let n = incomes.len();
    if n == 1 {
        return Ok(0.0);
    }

    let mut sorted: Vec<f64> = incomes.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));

    let total: f64 = sorted.iter().sum();
    if total.abs() < f64::EPSILON {
        return Ok(0.0);
    }

    let mut sum_weighted = 0.0;
    for (i, &val) in sorted.iter().enumerate() {
        sum_weighted += (i as f64 + 1.0) * val;
    }

    let n_f = n as f64;
    let gini = (2.0 * sum_weighted) / (n_f * total) - (n_f + 1.0) / n_f;
    Ok(gini)
}

/// Lorenz curve: cumulative share of income by cumulative share of population.
///
/// Returns a vector of (population_fraction, income_fraction) points from (0,0) to (1,1).
///
/// # Errors
///
/// Returns error if incomes is empty.
#[must_use = "returns the Lorenz curve points without side effects"]
pub fn lorenz_curve(incomes: &[f64]) -> Result<Vec<(f64, f64)>> {
    if incomes.is_empty() {
        return Err(SanghaError::ComputationError(
            "need at least one income value".into(),
        ));
    }

    let mut sorted: Vec<f64> = incomes.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));

    let total: f64 = sorted.iter().sum();
    let n = sorted.len() as f64;

    let mut points = Vec::with_capacity(sorted.len() + 1);
    points.push((0.0, 0.0));

    let mut cumulative = 0.0;
    for (i, &val) in sorted.iter().enumerate() {
        cumulative += val;
        let pop_frac = (i as f64 + 1.0) / n;
        let income_frac = if total > 0.0 {
            cumulative / total
        } else {
            pop_frac
        };
        points.push((pop_frac, income_frac));
    }

    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gini_perfect_equality() {
        let incomes = vec![100.0, 100.0, 100.0, 100.0, 100.0];
        let g = gini_coefficient(&incomes).unwrap();
        assert!(g.abs() < 1e-10);
    }

    #[test]
    fn test_gini_high_inequality() {
        // One person has everything
        let incomes = vec![0.0, 0.0, 0.0, 0.0, 1000.0];
        let g = gini_coefficient(&incomes).unwrap();
        assert!(g > 0.7); // very unequal
    }

    #[test]
    fn test_gini_moderate() {
        let incomes = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let g = gini_coefficient(&incomes).unwrap();
        assert!(g > 0.0);
        assert!(g < 0.5);
    }

    #[test]
    fn test_gini_empty() {
        assert!(gini_coefficient(&[]).is_err());
    }

    #[test]
    fn test_gini_single() {
        let g = gini_coefficient(&[100.0]).unwrap();
        assert!((g - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_lorenz_curve_shape() {
        let incomes = vec![10.0, 20.0, 30.0, 40.0];
        let curve = lorenz_curve(&incomes).unwrap();
        assert_eq!(curve.len(), 5); // 4 points + origin
        assert_eq!(curve[0], (0.0, 0.0));
        let last = curve.last().unwrap();
        assert!((last.0 - 1.0).abs() < 1e-10);
        assert!((last.1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_lorenz_curve_equality() {
        let incomes = vec![25.0, 25.0, 25.0, 25.0];
        let curve = lorenz_curve(&incomes).unwrap();
        // With perfect equality, Lorenz curve = 45-degree line
        for &(pop, income) in &curve {
            assert!((pop - income).abs() < 1e-10);
        }
    }
}
