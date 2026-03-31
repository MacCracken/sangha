//! Emotional and behavioral contagion — Hatfield model, SIS dynamics, mood propagation.

use serde::{Deserialize, Serialize};

use crate::error::{
    Result, SanghaError, validate_finite, validate_non_negative, validate_positive,
};

/// Emotional state of an agent.
///
/// Deserialization validates invariants automatically.
#[derive(Debug, Clone, Copy, Serialize)]
#[non_exhaustive]
pub struct EmotionalState {
    /// Valence: 0.0 (very negative) to 1.0 (very positive).
    pub valence: f64,
    /// Susceptibility to contagion: 0.0 (immune) to 1.0 (fully susceptible).
    pub susceptibility: f64,
}

impl<'de> Deserialize<'de> for EmotionalState {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> core::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            valence: f64,
            susceptibility: f64,
        }
        let raw = Raw::deserialize(deserializer)?;
        EmotionalState::new(raw.valence, raw.susceptibility).map_err(serde::de::Error::custom)
    }
}

impl EmotionalState {
    /// Create a new emotional state.
    ///
    /// # Errors
    ///
    /// Returns error if values are non-finite or outside `[0, 1]`.
    pub fn new(valence: f64, susceptibility: f64) -> Result<Self> {
        validate_finite(valence, "valence")?;
        validate_finite(susceptibility, "susceptibility")?;
        if !(0.0..=1.0).contains(&valence) {
            return Err(SanghaError::ComputationError(format!(
                "valence must be in [0, 1], got {valence}"
            )));
        }
        if !(0.0..=1.0).contains(&susceptibility) {
            return Err(SanghaError::ComputationError(format!(
                "susceptibility must be in [0, 1], got {susceptibility}"
            )));
        }
        Ok(Self {
            valence,
            susceptibility,
        })
    }

    /// Validate that this state is well-formed.
    ///
    /// Call this after deserialization to ensure invariants hold.
    ///
    /// # Errors
    ///
    /// Returns error if values are non-finite or outside `[0, 1]`.
    pub fn validate(&self) -> Result<()> {
        validate_finite(self.valence, "valence")?;
        validate_finite(self.susceptibility, "susceptibility")?;
        if !(0.0..=1.0).contains(&self.valence) {
            return Err(SanghaError::ComputationError(format!(
                "valence must be in [0, 1], got {}",
                self.valence
            )));
        }
        if !(0.0..=1.0).contains(&self.susceptibility) {
            return Err(SanghaError::ComputationError(format!(
                "susceptibility must be in [0, 1], got {}",
                self.susceptibility
            )));
        }
        Ok(())
    }
}

/// SIS (Susceptible-Infected-Susceptible) compartmental state.
///
/// Unlike SIR, infected individuals return to susceptible (behavioral relapse).
///
/// Deserialization validates invariants automatically.
#[derive(Debug, Clone, Copy, Serialize)]
#[non_exhaustive]
pub struct SisState {
    /// Fraction susceptible.
    pub s: f64,
    /// Fraction infected (exhibiting behavior).
    pub i: f64,
}

impl<'de> Deserialize<'de> for SisState {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> core::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            s: f64,
            i: f64,
        }
        let raw = Raw::deserialize(deserializer)?;
        SisState::new(raw.s, raw.i).map_err(serde::de::Error::custom)
    }
}

impl SisState {
    /// Create a new SIS state.
    ///
    /// # Errors
    ///
    /// Returns error if `s` or `i` are negative, non-finite, or `s + i` deviates
    /// from 1.0 by more than 1e-6.
    pub fn new(s: f64, i: f64) -> Result<Self> {
        validate_non_negative(s, "s")?;
        validate_non_negative(i, "i")?;
        let total = s + i;
        if (total - 1.0).abs() > 1e-6 {
            return Err(SanghaError::ComputationError(format!(
                "s + i must equal 1.0, got {total}"
            )));
        }
        Ok(Self { s, i })
    }
}

/// Configuration for the Hatfield emotional contagion model.
///
/// Deserialization validates invariants automatically.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct HatfieldConfig {
    /// Rate of emotional mimicry (0.0 to 1.0).
    pub mimicry_rate: f64,
    /// Feedback strength: how much the neighbor-weighted average emotion feeds
    /// back to the agent's felt emotion (0.0 = no feedback, 1.0 = strong).
    pub feedback_strength: f64,
}

impl<'de> Deserialize<'de> for HatfieldConfig {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> core::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            mimicry_rate: f64,
            feedback_strength: f64,
        }
        let raw = Raw::deserialize(deserializer)?;
        HatfieldConfig::new(raw.mimicry_rate, raw.feedback_strength)
            .map_err(serde::de::Error::custom)
    }
}

impl HatfieldConfig {
    /// Create a new Hatfield contagion configuration.
    ///
    /// # Errors
    ///
    /// Returns error if values are non-finite or negative.
    pub fn new(mimicry_rate: f64, feedback_strength: f64) -> Result<Self> {
        validate_non_negative(mimicry_rate, "mimicry_rate")?;
        validate_non_negative(feedback_strength, "feedback_strength")?;
        Ok(Self {
            mimicry_rate,
            feedback_strength,
        })
    }

    /// Validate that this config is well-formed.
    ///
    /// Call this after deserialization to ensure invariants hold.
    ///
    /// # Errors
    ///
    /// Returns error if values are non-finite or negative.
    pub fn validate(&self) -> Result<()> {
        validate_non_negative(self.mimicry_rate, "mimicry_rate")?;
        validate_non_negative(self.feedback_strength, "feedback_strength")
    }
}

/// One step of the Hatfield emotional contagion model on a weighted network.
///
/// For each agent `i`:
/// ```text
/// mimicry  = susceptibility_i * mimicry_rate * Σ_j w_ij * (valence_j - valence_i)
/// feedback = feedback_strength * (expressed_i - valence_i)
/// valence_i' = valence_i + dt * (mimicry + feedback)
/// ```
///
/// where `expressed_i` is the weight-averaged neighbor valence.
///
/// The adjacency list is `&[Vec<(usize, f64)>]` where `adjacency[i]` contains
/// `(neighbor_index, weight)` pairs. This avoids importing `network::SocialNetwork`.
///
/// # Errors
///
/// Returns error if `states` and `adjacency` lengths differ, or indices are out of bounds.
#[must_use = "returns the updated emotional states without side effects"]
pub fn hatfield_contagion_step(
    states: &[EmotionalState],
    adjacency: &[Vec<(usize, f64)>],
    config: &HatfieldConfig,
    dt: f64,
) -> Result<Vec<EmotionalState>> {
    validate_positive(dt, "dt")?;
    if states.len() != adjacency.len() {
        return Err(SanghaError::ComputationError(format!(
            "states length {} != adjacency length {}",
            states.len(),
            adjacency.len()
        )));
    }

    let n = states.len();
    let mut new_states = Vec::with_capacity(n);

    for (i, state) in states.iter().enumerate() {
        let mut influence_sum = 0.0;
        for &(j, w) in &adjacency[i] {
            if j >= n {
                return Err(SanghaError::InvalidNetwork(format!(
                    "neighbor index {j} out of bounds for {n} agents"
                )));
            }
            influence_sum += w * (states[j].valence - state.valence);
        }

        // Mimicry: agent moves toward weighted average of neighbors
        let mimicry = state.susceptibility * config.mimicry_rate * influence_sum;
        // Feedback: expressed emotion (influenced by neighbors) feeds back to felt emotion
        // expressed_i approximated as the neighbor-weighted mean influence on i
        let expressed = if adjacency[i].is_empty() {
            state.valence
        } else {
            let total_w: f64 = adjacency[i].iter().map(|&(_, w)| w).sum();
            if total_w > 0.0 {
                adjacency[i]
                    .iter()
                    .map(|&(j, w)| w * states[j].valence)
                    .sum::<f64>()
                    / total_w
            } else {
                state.valence
            }
        };
        let feedback = config.feedback_strength * (expressed - state.valence);

        let dv = mimicry + feedback;
        let new_valence = (state.valence + dt * dv).clamp(0.0, 1.0);

        new_states.push(EmotionalState {
            valence: new_valence,
            susceptibility: state.susceptibility,
        });
    }

    Ok(new_states)
}

/// One step of the SIS epidemiological model.
///
/// - `dS/dt = -beta * S * I + gamma * I`
/// - `dI/dt = beta * S * I - gamma * I`
///
/// Unlike SIR, recovered individuals return to susceptible.
///
/// # Errors
///
/// Returns error if parameters are invalid.
#[inline]
#[must_use = "returns the new SIS state without side effects"]
pub fn sis_step(s: f64, i: f64, beta: f64, gamma: f64, dt: f64) -> Result<SisState> {
    validate_non_negative(s, "s")?;
    validate_non_negative(i, "i")?;
    validate_positive(beta, "beta")?;
    validate_positive(gamma, "gamma")?;
    validate_positive(dt, "dt")?;

    let ds = -beta * s * i + gamma * i;
    let di = beta * s * i - gamma * i;

    let new_s = (s + ds * dt).max(0.0);
    let new_i = (i + di * dt).max(0.0);

    // Renormalize to preserve S + I = 1 invariant
    let total = new_s + new_i;
    if total > 0.0 {
        Ok(SisState {
            s: new_s / total,
            i: new_i / total,
        })
    } else {
        Ok(SisState { s: 1.0, i: 0.0 })
    }
}

/// SIS endemic equilibrium: steady-state infection fraction.
///
/// When `beta > gamma`: `I* = 1 - gamma/beta`
/// When `beta <= gamma`: `I* = 0` (disease dies out)
///
/// # Errors
///
/// Returns error if parameters are non-positive.
#[inline]
#[must_use = "returns the endemic equilibrium without side effects"]
pub fn sis_endemic_equilibrium(beta: f64, gamma: f64) -> Result<f64> {
    validate_positive(beta, "beta")?;
    validate_positive(gamma, "gamma")?;
    if beta > gamma {
        Ok(1.0 - gamma / beta)
    } else {
        Ok(0.0)
    }
}

/// Simplified mood propagation via linear diffusion with decay toward neutral.
///
/// For each agent `i`:
/// ```text
/// mood_i' = mood_i + dt * Σ_j w_ij * (mood_j - mood_i) - dt * decay * (mood_i - 0.5)
/// ```
///
/// The decay term pulls moods toward neutral (0.5).
///
/// # Errors
///
/// Returns error if `moods` and `adjacency` lengths differ, or parameters are invalid.
#[must_use = "returns the updated moods without side effects"]
pub fn mood_propagation(
    moods: &[f64],
    adjacency: &[Vec<(usize, f64)>],
    decay: f64,
    dt: f64,
) -> Result<Vec<f64>> {
    validate_non_negative(decay, "decay")?;
    validate_positive(dt, "dt")?;
    if moods.len() != adjacency.len() {
        return Err(SanghaError::ComputationError(format!(
            "moods length {} != adjacency length {}",
            moods.len(),
            adjacency.len()
        )));
    }

    let n = moods.len();
    let mut new_moods = Vec::with_capacity(n);

    for (i, &mood) in moods.iter().enumerate() {
        validate_finite(mood, &format!("moods[{i}]"))?;

        let mut diffusion = 0.0;
        for &(j, w) in &adjacency[i] {
            if j >= n {
                return Err(SanghaError::InvalidNetwork(format!(
                    "neighbor index {j} out of bounds for {n} agents"
                )));
            }
            diffusion += w * (moods[j] - mood);
        }

        let decay_term = decay * (mood - 0.5);
        let new_mood = mood + dt * diffusion - dt * decay_term;
        new_moods.push(new_mood.clamp(0.0, 1.0));
    }

    Ok(new_moods)
}

/// Epidemic threshold via power iteration on the adjacency matrix.
///
/// Returns `beta_c = 1 / lambda_max` where `lambda_max` is the largest
/// eigenvalue of the adjacency matrix. Below this threshold, epidemics die out.
///
/// Uses power iteration (max 1000 iterations, tolerance 1e-10).
///
/// # Errors
///
/// Returns error if the network is empty or power iteration fails to converge.
#[must_use = "returns the epidemic threshold without side effects"]
pub fn contagion_threshold(adjacency: &[Vec<(usize, f64)>]) -> Result<f64> {
    let n = adjacency.len();
    if n == 0 {
        return Err(SanghaError::ComputationError("empty network".into()));
    }

    // Power iteration to find largest eigenvalue
    let mut v = vec![1.0 / (n as f64).sqrt(); n];
    let mut lambda = 0.0;
    let max_iter = 1000;
    let tol = 1e-10;

    for _ in 0..max_iter {
        // Matrix-vector multiply: w = A * v
        let mut w = vec![0.0; n];
        for (i, neighbors) in adjacency.iter().enumerate() {
            for &(j, weight) in neighbors {
                if j >= n {
                    return Err(SanghaError::InvalidNetwork(format!(
                        "neighbor index {j} out of bounds for {n} agents"
                    )));
                }
                w[i] += weight * v[j];
            }
        }

        // Compute norm
        let norm: f64 = w.iter().map(|&x| x * x).sum::<f64>().sqrt();
        if norm < f64::EPSILON {
            // Zero matrix: no epidemic possible
            return Err(SanghaError::SimulationFailed(
                "adjacency matrix has zero spectral radius".into(),
            ));
        }

        let new_lambda = norm;
        // Normalize
        for x in &mut w {
            *x /= norm;
        }

        if (new_lambda - lambda).abs() < tol {
            return Ok(1.0 / new_lambda);
        }

        lambda = new_lambda;
        v = w;
    }

    Err(SanghaError::SimulationFailed(
        "power iteration did not converge after 1000 iterations".into(),
    ))
}

/// Check if all emotional states have converged (all valences within `epsilon`).
///
/// # Errors
///
/// Returns error if `epsilon` is non-positive.
#[inline]
#[must_use = "returns convergence status without side effects"]
pub fn emotional_convergence(states: &[EmotionalState], epsilon: f64) -> Result<bool> {
    validate_positive(epsilon, "epsilon")?;
    if states.len() <= 1 {
        return Ok(true);
    }
    let mean: f64 = states.iter().map(|s| s.valence).sum::<f64>() / states.len() as f64;
    Ok(states.iter().all(|s| (s.valence - mean).abs() < epsilon))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- EmotionalState ---

    #[test]
    fn test_emotional_state_valid() {
        let s = EmotionalState::new(0.5, 0.8).unwrap();
        assert!((s.valence - 0.5).abs() < 1e-10);
        assert!((s.susceptibility - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_emotional_state_out_of_range() {
        assert!(EmotionalState::new(1.5, 0.5).is_err());
        assert!(EmotionalState::new(0.5, -0.1).is_err());
    }

    #[test]
    fn test_emotional_state_nan() {
        assert!(EmotionalState::new(f64::NAN, 0.5).is_err());
    }

    // --- hatfield_contagion_step ---

    #[test]
    fn test_hatfield_convergence() {
        // Two connected agents: one happy (0.8), one sad (0.2)
        let states = vec![
            EmotionalState::new(0.8, 1.0).unwrap(),
            EmotionalState::new(0.2, 1.0).unwrap(),
        ];
        let adj = vec![vec![(1, 1.0)], vec![(0, 1.0)]];
        let config = HatfieldConfig::new(0.5, 0.0).unwrap();

        let new = hatfield_contagion_step(&states, &adj, &config, 0.1).unwrap();
        // Should converge toward each other
        assert!(new[0].valence < 0.8);
        assert!(new[1].valence > 0.2);
    }

    #[test]
    fn test_hatfield_isolated_no_change() {
        let states = vec![EmotionalState::new(0.5, 1.0).unwrap()];
        let adj: Vec<Vec<(usize, f64)>> = vec![vec![]];
        let config = HatfieldConfig::new(0.5, 0.0).unwrap();

        let new = hatfield_contagion_step(&states, &adj, &config, 0.1).unwrap();
        assert!((new[0].valence - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_hatfield_zero_susceptibility() {
        let states = vec![
            EmotionalState::new(0.8, 0.0).unwrap(), // immune
            EmotionalState::new(0.2, 1.0).unwrap(),
        ];
        let adj = vec![vec![(1, 1.0)], vec![(0, 1.0)]];
        let config = HatfieldConfig::new(1.0, 0.0).unwrap();

        let new = hatfield_contagion_step(&states, &adj, &config, 0.1).unwrap();
        // Agent 0 is immune: no change
        assert!((new[0].valence - 0.8).abs() < 1e-10);
        // Agent 1 should move toward 0.8
        assert!(new[1].valence > 0.2);
    }

    #[test]
    fn test_hatfield_uniform_no_change() {
        let states = vec![
            EmotionalState::new(0.5, 1.0).unwrap(),
            EmotionalState::new(0.5, 1.0).unwrap(),
            EmotionalState::new(0.5, 1.0).unwrap(),
        ];
        let adj = vec![vec![(1, 1.0), (2, 1.0)], vec![(0, 1.0)], vec![(0, 1.0)]];
        let config = HatfieldConfig::new(0.5, 0.0).unwrap();

        let new = hatfield_contagion_step(&states, &adj, &config, 0.1).unwrap();
        for s in &new {
            assert!((s.valence - 0.5).abs() < 1e-10);
        }
    }

    #[test]
    fn test_hatfield_length_mismatch() {
        let states = vec![EmotionalState::new(0.5, 1.0).unwrap()];
        let adj: Vec<Vec<(usize, f64)>> = vec![vec![], vec![]];
        let config = HatfieldConfig::new(0.5, 0.0).unwrap();
        assert!(hatfield_contagion_step(&states, &adj, &config, 0.1).is_err());
    }

    #[test]
    fn test_hatfield_out_of_bounds_neighbor() {
        let states = vec![EmotionalState::new(0.5, 1.0).unwrap()];
        let adj = vec![vec![(5, 1.0)]]; // index 5 doesn't exist
        let config = HatfieldConfig::new(0.5, 0.0).unwrap();
        assert!(hatfield_contagion_step(&states, &adj, &config, 0.1).is_err());
    }

    // --- SIS ---

    #[test]
    fn test_sis_conservation() {
        // S + I should stay approximately constant
        let state = sis_step(0.9, 0.1, 0.5, 0.2, 0.01).unwrap();
        let total = state.s + state.i;
        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_sis_declining_epidemic() {
        // beta < gamma → infection declines
        let state = sis_step(0.9, 0.1, 0.1, 0.5, 0.1).unwrap();
        assert!(state.i < 0.1);
    }

    #[test]
    fn test_sis_endemic_above_threshold() {
        // beta > gamma → endemic equilibrium is positive
        let eq = sis_endemic_equilibrium(0.5, 0.2).unwrap();
        assert!((eq - 0.6).abs() < 1e-10); // 1 - 0.2/0.5 = 0.6
    }

    #[test]
    fn test_sis_endemic_below_threshold() {
        let eq = sis_endemic_equilibrium(0.2, 0.5).unwrap();
        assert!((eq - 0.0).abs() < 1e-10);
    }

    // --- mood_propagation ---

    #[test]
    fn test_mood_propagation_converges() {
        let moods = vec![0.8, 0.2];
        let adj = vec![vec![(1, 1.0)], vec![(0, 1.0)]];

        let new = mood_propagation(&moods, &adj, 0.0, 0.1).unwrap();
        assert!(new[0] < 0.8);
        assert!(new[1] > 0.2);
    }

    #[test]
    fn test_mood_propagation_decay_to_neutral() {
        // Single agent, no neighbors, with decay → should move toward 0.5
        let moods = vec![0.9];
        let adj: Vec<Vec<(usize, f64)>> = vec![vec![]];

        let new = mood_propagation(&moods, &adj, 1.0, 0.1).unwrap();
        assert!(new[0] < 0.9); // decayed toward 0.5
    }

    #[test]
    fn test_mood_propagation_uniform() {
        let moods = vec![0.5, 0.5, 0.5];
        let adj = vec![vec![(1, 1.0)], vec![(0, 1.0), (2, 1.0)], vec![(1, 1.0)]];

        let new = mood_propagation(&moods, &adj, 0.0, 0.1).unwrap();
        for &m in &new {
            assert!((m - 0.5).abs() < 1e-10);
        }
    }

    // --- contagion_threshold ---

    #[test]
    fn test_contagion_threshold_complete_graph() {
        // Complete graph of 4 nodes, weight 1: lambda_max = 3 (n-1)
        // beta_c = 1/3
        let adj = vec![
            vec![(1, 1.0), (2, 1.0), (3, 1.0)],
            vec![(0, 1.0), (2, 1.0), (3, 1.0)],
            vec![(0, 1.0), (1, 1.0), (3, 1.0)],
            vec![(0, 1.0), (1, 1.0), (2, 1.0)],
        ];
        let threshold = contagion_threshold(&adj).unwrap();
        assert!((threshold - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_contagion_threshold_empty_error() {
        let adj: Vec<Vec<(usize, f64)>> = vec![];
        assert!(contagion_threshold(&adj).is_err());
    }

    // --- emotional_convergence ---

    #[test]
    fn test_emotional_convergence_true() {
        let states = vec![
            EmotionalState::new(0.5, 1.0).unwrap(),
            EmotionalState::new(0.5, 0.5).unwrap(),
        ];
        assert!(emotional_convergence(&states, 0.01).unwrap());
    }

    #[test]
    fn test_emotional_convergence_false() {
        let states = vec![
            EmotionalState::new(0.1, 1.0).unwrap(),
            EmotionalState::new(0.9, 1.0).unwrap(),
        ];
        assert!(!emotional_convergence(&states, 0.01).unwrap());
    }

    #[test]
    fn test_emotional_convergence_single() {
        let states = vec![EmotionalState::new(0.5, 1.0).unwrap()];
        assert!(emotional_convergence(&states, 0.01).unwrap());
    }

    #[test]
    fn test_emotional_convergence_invalid_epsilon() {
        let states = vec![EmotionalState::new(0.5, 1.0).unwrap()];
        assert!(emotional_convergence(&states, 0.0).is_err());
    }

    // --- serde roundtrips ---

    #[test]
    fn test_emotional_state_serde_roundtrip() {
        let s = EmotionalState::new(0.7, 0.3).unwrap();
        let json = serde_json::to_string(&s).unwrap();
        let back: EmotionalState = serde_json::from_str(&json).unwrap();
        assert!((s.valence - back.valence).abs() < 1e-10);
    }

    #[test]
    fn test_sis_state_serde_roundtrip() {
        let s = SisState::new(0.9, 0.1).unwrap();
        let json = serde_json::to_string(&s).unwrap();
        let back: SisState = serde_json::from_str(&json).unwrap();
        assert!((s.s - back.s).abs() < 1e-10);
    }

    #[test]
    fn test_hatfield_config_serde_roundtrip() {
        let c = HatfieldConfig::new(0.5, 0.3).unwrap();
        let json = serde_json::to_string(&c).unwrap();
        let back: HatfieldConfig = serde_json::from_str(&json).unwrap();
        assert!((c.mimicry_rate - back.mimicry_rate).abs() < 1e-10);
    }

    // --- audit tests ---

    #[test]
    fn test_sis_step_clamp_negative() {
        // Large dt forces negative intermediate → clamped to 0, renormalized
        let state = sis_step(0.01, 0.99, 0.5, 10.0, 1.0).unwrap();
        assert!(state.s >= 0.0);
        assert!(state.i >= 0.0);
        assert!((state.s + state.i - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_sis_state_invalid() {
        assert!(SisState::new(0.5, 0.6).is_err()); // sum > 1
        assert!(SisState::new(-0.1, 1.1).is_err()); // negative
        assert!(SisState::new(f64::NAN, 0.5).is_err());
    }

    #[test]
    fn test_contagion_threshold_disconnected() {
        // Node 0 connected to 1, node 2 isolated
        let adj = vec![vec![(1, 1.0)], vec![(0, 1.0)], vec![]];
        // Should still converge (largest eigenvalue from connected component)
        let threshold = contagion_threshold(&adj).unwrap();
        assert!(threshold > 0.0);
    }

    #[test]
    fn test_mood_propagation_oob_error() {
        let moods = vec![0.5];
        let adj = vec![vec![(5, 1.0)]]; // OOB
        assert!(mood_propagation(&moods, &adj, 0.0, 0.1).is_err());
    }

    #[test]
    fn test_hatfield_dt_zero_error() {
        let states = vec![EmotionalState::new(0.5, 1.0).unwrap()];
        let adj: Vec<Vec<(usize, f64)>> = vec![vec![]];
        let config = HatfieldConfig::new(0.5, 0.0).unwrap();
        assert!(hatfield_contagion_step(&states, &adj, &config, 0.0).is_err());
    }

    #[test]
    fn test_mood_propagation_single_no_decay() {
        let moods = vec![0.7];
        let adj: Vec<Vec<(usize, f64)>> = vec![vec![]];
        let new = mood_propagation(&moods, &adj, 0.0, 0.1).unwrap();
        assert!((new[0] - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_contagion_threshold_oob_error() {
        let adj = vec![vec![(5, 1.0)]]; // OOB neighbor
        assert!(contagion_threshold(&adj).is_err());
    }

    #[test]
    fn test_hatfield_feedback_strength() {
        // With high feedback_strength, agent should move more toward neighbor average
        let states = vec![
            EmotionalState::new(0.2, 1.0).unwrap(),
            EmotionalState::new(0.8, 1.0).unwrap(),
        ];
        let adj = vec![vec![(1, 1.0)], vec![(0, 1.0)]];
        let no_feedback = HatfieldConfig::new(0.5, 0.0).unwrap();
        let with_feedback = HatfieldConfig::new(0.5, 0.5).unwrap();

        let new_no = hatfield_contagion_step(&states, &adj, &no_feedback, 0.1).unwrap();
        let new_yes = hatfield_contagion_step(&states, &adj, &with_feedback, 0.1).unwrap();

        // With feedback, agent 0 should move more toward agent 1
        assert!(new_yes[0].valence > new_no[0].valence);
    }

    #[test]
    fn test_emotional_state_deserialize_rejects_invalid() {
        // valence > 1.0 is invalid
        let json = r#"{"valence":1.5,"susceptibility":0.5}"#;
        let result: core::result::Result<EmotionalState, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_sis_state_deserialize_rejects_invalid() {
        // s + i != 1
        let json = r#"{"s":0.5,"i":0.8}"#;
        let result: core::result::Result<SisState, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_hatfield_config_deserialize_rejects_invalid() {
        // negative mimicry_rate
        let json = r#"{"mimicry_rate":-0.5,"feedback_strength":0.1}"#;
        let result: core::result::Result<HatfieldConfig, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
