//! Population dynamics — growth models, epidemiology (SIR).

use serde::{Deserialize, Serialize};

use crate::error::{validate_finite, validate_non_negative, validate_positive, Result};

/// Logistic growth: population change with carrying capacity.
///
/// `dN/dt = r * N * (1 - N/K)`
///
/// Returns the new population after time step `dt`.
///
/// # Errors
///
/// Returns error if parameters are invalid.
#[inline]
#[must_use = "returns the population change without side effects"]
pub fn logistic_growth(n: f64, r: f64, k: f64) -> Result<f64> {
    validate_non_negative(n, "n")?;
    validate_finite(r, "r")?;
    validate_positive(k, "k")?;
    Ok(r * n * (1.0 - n / k))
}

/// Logistic growth step: advance population by `dt`.
///
/// # Errors
///
/// Returns error if parameters are invalid.
#[inline]
#[must_use = "returns the new population without side effects"]
pub fn logistic_growth_step(n: f64, r: f64, k: f64, dt: f64) -> Result<f64> {
    let dn = logistic_growth(n, r, k)?;
    validate_positive(dt, "dt")?;
    Ok(n + dn * dt)
}

/// SIR compartmental model state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SirState {
    /// Susceptible fraction (0-1).
    pub s: f64,
    /// Infected fraction (0-1).
    pub i: f64,
    /// Recovered fraction (0-1).
    pub r: f64,
}

/// One step of the SIR epidemiological model.
///
/// - `dS/dt = -beta * S * I`
/// - `dI/dt = beta * S * I - gamma * I`
/// - `dR/dt = gamma * I`
///
/// # Errors
///
/// Returns error if parameters are invalid.
#[must_use = "returns the new SIR state without side effects"]
pub fn sir_step(s: f64, i: f64, r: f64, beta: f64, gamma: f64, dt: f64) -> Result<SirState> {
    validate_non_negative(s, "s")?;
    validate_non_negative(i, "i")?;
    validate_non_negative(r, "r")?;
    validate_positive(beta, "beta")?;
    validate_positive(gamma, "gamma")?;
    validate_positive(dt, "dt")?;

    let ds = -beta * s * i;
    let di = beta * s * i - gamma * i;
    let dr = gamma * i;

    Ok(SirState {
        s: (s + ds * dt).max(0.0),
        i: (i + di * dt).max(0.0),
        r: (r + dr * dt).max(0.0),
    })
}

/// Basic reproduction number R0.
///
/// `R0 = beta / gamma`
///
/// R0 > 1: epidemic grows. R0 < 1: epidemic dies out.
///
/// # Errors
///
/// Returns error if gamma is non-positive.
#[inline]
#[must_use = "returns R0 without side effects"]
pub fn r_naught(beta: f64, gamma: f64) -> Result<f64> {
    validate_positive(beta, "beta")?;
    validate_positive(gamma, "gamma")?;
    Ok(beta / gamma)
}

/// Herd immunity threshold.
///
/// `H = 1 - 1/R0`
///
/// The fraction of the population that must be immune to prevent epidemic spread.
///
/// # Errors
///
/// Returns error if R0 is non-positive.
#[inline]
#[must_use = "returns the herd immunity threshold without side effects"]
pub fn herd_immunity_threshold(r0: f64) -> Result<f64> {
    validate_positive(r0, "r0")?;
    Ok(1.0 - 1.0 / r0)
}

/// Run a full SIR trajectory over a given duration.
///
/// # Errors
///
/// Returns error if parameters are invalid.
#[must_use = "returns the SIR trajectory without side effects"]
pub fn sir_trajectory(
    s0: f64,
    i0: f64,
    r0: f64,
    beta: f64,
    gamma: f64,
    dt: f64,
    steps: usize,
) -> Result<alloc::vec::Vec<SirState>> {
    extern crate alloc;
    let mut trajectory = alloc::vec::Vec::with_capacity(steps + 1);
    trajectory.push(SirState {
        s: s0,
        i: i0,
        r: r0,
    });

    let mut state = SirState {
        s: s0,
        i: i0,
        r: r0,
    };
    for _ in 0..steps {
        state = sir_step(state.s, state.i, state.r, beta, gamma, dt)?;
        trajectory.push(state);
    }

    Ok(trajectory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logistic_growth_at_zero() {
        let dn = logistic_growth(0.0, 0.5, 100.0).unwrap();
        assert!((dn - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_logistic_growth_at_capacity() {
        let dn = logistic_growth(100.0, 0.5, 100.0).unwrap();
        assert!((dn - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_logistic_growth_max_at_half_k() {
        // Maximum growth rate occurs at N = K/2
        let dn_half = logistic_growth(50.0, 1.0, 100.0).unwrap();
        let dn_quarter = logistic_growth(25.0, 1.0, 100.0).unwrap();
        let dn_three_quarter = logistic_growth(75.0, 1.0, 100.0).unwrap();
        assert!(dn_half > dn_quarter);
        assert!(dn_half > dn_three_quarter);
    }

    #[test]
    fn test_r_naught() {
        let r0 = r_naught(0.5, 0.2).unwrap();
        assert!((r0 - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_herd_immunity_r0_3() {
        // R0 = 3: herd immunity = 1 - 1/3 = 0.667
        let h = herd_immunity_threshold(3.0).unwrap();
        assert!((h - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_herd_immunity_r0_2_5() {
        let h = herd_immunity_threshold(2.5).unwrap();
        assert!((h - 0.6).abs() < 1e-10);
    }

    #[test]
    fn test_sir_step_conservation() {
        // S + I + R should be approximately conserved
        let state = sir_step(0.99, 0.01, 0.0, 0.5, 0.1, 0.01).unwrap();
        let total = state.s + state.i + state.r;
        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_sir_declining_epidemic() {
        // With R0 < 1 (beta=0.1, gamma=0.5, R0=0.2), infection should decline
        let state = sir_step(0.99, 0.01, 0.0, 0.1, 0.5, 0.1).unwrap();
        assert!(state.i < 0.01);
    }

    #[test]
    fn test_sir_state_serde_roundtrip() {
        let state = SirState {
            s: 0.9,
            i: 0.05,
            r: 0.05,
        };
        let json = serde_json::to_string(&state).unwrap();
        let back: SirState = serde_json::from_str(&json).unwrap();
        assert!((state.s - back.s).abs() < 1e-10);
    }
}
