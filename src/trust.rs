//! Trust and reputation — trust propagation, reputation aggregation, decay, betrayal.

use serde::{Deserialize, Serialize};

use crate::error::{Result, SanghaError, validate_finite, validate_non_negative};

/// A directed trust relationship between two agents.
///
/// `trust_level` ranges from -1.0 (complete distrust) to 1.0 (complete trust).
///
/// Deserialization validates invariants automatically.
#[derive(Debug, Clone, Copy, Serialize)]
#[non_exhaustive]
pub struct TrustRelation {
    /// Index of the trusting agent.
    pub truster: usize,
    /// Index of the trusted agent.
    pub trustee: usize,
    /// Trust level in \[-1, 1\].
    pub trust_level: f64,
}

impl<'de> Deserialize<'de> for TrustRelation {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> core::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            truster: usize,
            trustee: usize,
            trust_level: f64,
        }
        let raw = Raw::deserialize(deserializer)?;
        TrustRelation::new(raw.truster, raw.trustee, raw.trust_level)
            .map_err(serde::de::Error::custom)
    }
}

impl TrustRelation {
    /// Create a new trust relation.
    ///
    /// # Errors
    ///
    /// Returns error if `trust_level` is not in \[-1, 1\] or not finite.
    pub fn new(truster: usize, trustee: usize, trust_level: f64) -> Result<Self> {
        validate_finite(trust_level, "trust_level")?;
        if !(-1.0..=1.0).contains(&trust_level) {
            return Err(SanghaError::ComputationError(format!(
                "trust_level must be in [-1, 1], got {trust_level}"
            )));
        }
        Ok(Self {
            truster,
            trustee,
            trust_level,
        })
    }
}

/// A directed weighted network representing trust between agents.
///
/// Unlike [`crate::network::SocialNetwork`], trust is asymmetric:
/// agent A trusting agent B does not imply B trusts A.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TrustNetwork {
    /// Number of agents.
    pub agent_count: usize,
    /// Directed adjacency list: `relations[i]` contains `(target, trust_level)` pairs.
    pub relations: Vec<Vec<(usize, f64)>>,
}

impl TrustNetwork {
    /// Create an empty trust network with `n` agents.
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            agent_count: n,
            relations: (0..n).map(|_| Vec::new()).collect(),
        }
    }

    /// Add a directed trust relation from `truster` to `trustee`.
    ///
    /// # Errors
    ///
    /// Returns error if indices are out of bounds or `trust_level` not in \[-1, 1\].
    pub fn add_trust(&mut self, truster: usize, trustee: usize, trust_level: f64) -> Result<()> {
        if truster >= self.agent_count || trustee >= self.agent_count {
            return Err(SanghaError::ComputationError(
                "agent index out of bounds".into(),
            ));
        }
        validate_finite(trust_level, "trust_level")?;
        if !(-1.0..=1.0).contains(&trust_level) {
            return Err(SanghaError::ComputationError(format!(
                "trust_level must be in [-1, 1], got {trust_level}"
            )));
        }
        self.relations[truster].push((trustee, trust_level));
        Ok(())
    }

    /// Get trust level from `truster` to `trustee`, or `None` if no relation exists.
    ///
    /// # Errors
    ///
    /// Returns error if indices are out of bounds.
    #[must_use = "returns the trust level without side effects"]
    pub fn get_trust(&self, truster: usize, trustee: usize) -> Result<Option<f64>> {
        if truster >= self.agent_count || trustee >= self.agent_count {
            return Err(SanghaError::ComputationError(
                "agent index out of bounds".into(),
            ));
        }
        Ok(self.relations[truster]
            .iter()
            .find(|&&(t, _)| t == trustee)
            .map(|&(_, level)| level))
    }

    /// Validate that this network is well-formed.
    ///
    /// Call this after deserialization to ensure invariants hold.
    ///
    /// # Errors
    ///
    /// Returns error if relations length mismatches agent_count, or any trust
    /// level is out of range or non-finite.
    pub fn validate(&self) -> Result<()> {
        if self.relations.len() != self.agent_count {
            return Err(SanghaError::ComputationError(format!(
                "relations length {} != agent_count {}",
                self.relations.len(),
                self.agent_count
            )));
        }
        for (i, neighbors) in self.relations.iter().enumerate() {
            for &(target, level) in neighbors {
                if target >= self.agent_count {
                    return Err(SanghaError::ComputationError(format!(
                        "relations[{i}] has target {target} out of bounds"
                    )));
                }
                validate_finite(level, &format!("relations[{i}] trust_level"))?;
                if !(-1.0..=1.0).contains(&level) {
                    return Err(SanghaError::ComputationError(format!(
                        "trust_level must be in [-1, 1], got {level}"
                    )));
                }
            }
        }
        Ok(())
    }
}

/// Aggregated reputation score for an agent.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReputationScore {
    /// Agent index.
    pub agent: usize,
    /// Reputation score (typically in \[-1, 1\]).
    pub score: f64,
}

impl ReputationScore {
    /// Create a new reputation score.
    #[inline]
    #[must_use]
    pub fn new(agent: usize, score: f64) -> Self {
        Self { agent, score }
    }
}

/// Exponential decay of trust over time without reinforcement.
///
/// `decayed_trust = trust_level * exp(-decay_rate * time_steps)`
///
/// # Errors
///
/// Returns error if `trust_level` not in \[-1, 1\], or `time_steps` or
/// `decay_rate` are negative or non-finite.
///
/// Reference: Golbeck (2005), *Computing and Applying Trust in Web-based Social Networks*.
#[inline]
#[must_use = "returns the decayed trust without side effects"]
pub fn trust_decay(trust_level: f64, time_steps: f64, decay_rate: f64) -> Result<f64> {
    validate_finite(trust_level, "trust_level")?;
    validate_non_negative(time_steps, "time_steps")?;
    validate_non_negative(decay_rate, "decay_rate")?;
    if !(-1.0..=1.0).contains(&trust_level) {
        return Err(SanghaError::ComputationError(format!(
            "trust_level must be in [-1, 1], got {trust_level}"
        )));
    }
    Ok(trust_level * (-decay_rate * time_steps).exp())
}

/// Compute trust after a betrayal event.
///
/// `new_trust = current_trust * (1 - severity)`
///
/// Captures the asymmetry of trust: hard to build, easy to destroy.
///
/// # Errors
///
/// Returns error if `current_trust` not in \[-1, 1\] or `severity` not in \[0, 1\].
#[inline]
#[must_use = "returns the new trust level without side effects"]
pub fn betrayal_impact(current_trust: f64, severity: f64) -> Result<f64> {
    validate_finite(current_trust, "current_trust")?;
    validate_finite(severity, "severity")?;
    if !(-1.0..=1.0).contains(&current_trust) {
        return Err(SanghaError::ComputationError(format!(
            "current_trust must be in [-1, 1], got {current_trust}"
        )));
    }
    if !(0.0..=1.0).contains(&severity) {
        return Err(SanghaError::ComputationError(format!(
            "severity must be in [0, 1], got {severity}"
        )));
    }
    Ok(current_trust * (1.0 - severity))
}

/// Aggregate reputation of a target agent from all who have a trust relation to it.
///
/// Reputation is the weighted mean of trust opinions, weighted by absolute trust
/// (stronger opinions count more):
///
/// `R(t) = Σ trust(i,t) * |trust(i,t)| / Σ |trust(i,t)|`
///
/// Returns score 0.0 if no agent has a trust relation to the target.
///
/// # Errors
///
/// Returns error if `target` >= `agent_count`.
///
/// Reference: Resnick et al. (2000), *Reputation Systems*, CACM 43(12).
#[must_use = "returns the reputation score without side effects"]
pub fn reputation_aggregate(network: &TrustNetwork, target: usize) -> Result<ReputationScore> {
    if target >= network.agent_count {
        return Err(SanghaError::ComputationError(
            "target agent index out of bounds".into(),
        ));
    }

    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;

    for neighbors in &network.relations {
        for &(t, level) in neighbors {
            if t == target {
                let w = level.abs();
                weighted_sum += level * w;
                weight_total += w;
            }
        }
    }

    let score = if weight_total > 0.0 {
        weighted_sum / weight_total
    } else {
        0.0
    };

    Ok(ReputationScore::new(target, score))
}

/// Compute indirect trust from source to target via trust transitivity.
///
/// Uses discounted path product: for each path `P = (e_1, ..., e_k)` with `k <= max_hops`:
/// `path_trust(P) = Π trust(e_i) * decay^k`
///
/// Returns the maximum path trust over all paths of length <= `max_hops`.
/// Returns 0.0 if no path exists within the hop limit.
/// Returns 1.0 if `source == target` (implicit self-trust).
///
/// # Errors
///
/// Returns error if `source` or `target` >= `agent_count`, `max_hops` is 0,
/// or `decay` is not in (0, 1\].
///
/// Reference: Golbeck (2005), *Computing and Applying Trust in Web-based Social Networks*.
#[must_use = "returns the propagated trust without side effects"]
pub fn trust_propagation(
    network: &TrustNetwork,
    source: usize,
    target: usize,
    max_hops: usize,
    decay: f64,
) -> Result<f64> {
    if source >= network.agent_count || target >= network.agent_count {
        return Err(SanghaError::ComputationError(
            "agent index out of bounds".into(),
        ));
    }
    if max_hops == 0 {
        return Err(SanghaError::ComputationError("max_hops must be > 0".into()));
    }
    validate_finite(decay, "decay")?;
    if !(0.0..=1.0).contains(&decay) || decay == 0.0 {
        return Err(SanghaError::ComputationError(format!(
            "decay must be in (0, 1], got {decay}"
        )));
    }

    if source == target {
        return Ok(1.0);
    }

    // Bounded BFS tracking best accumulated trust product to each node
    let n = network.agent_count;
    let mut best_trust = vec![f64::NEG_INFINITY; n];
    best_trust[source] = 1.0;

    // frontier: (node, accumulated_trust_product)
    let mut current_frontier: Vec<(usize, f64)> = vec![(source, 1.0)];

    for hop in 0..max_hops {
        let mut next_frontier: Vec<(usize, f64)> = Vec::new();
        let hop_decay = decay.powi((hop + 1) as i32);

        for &(node, acc_trust) in &current_frontier {
            for &(neighbor, edge_trust) in &network.relations[node] {
                let new_trust = acc_trust * edge_trust;
                let decayed = new_trust * hop_decay;

                if decayed > best_trust[neighbor] {
                    best_trust[neighbor] = decayed;
                    next_frontier.push((neighbor, new_trust));
                }
            }
        }

        current_frontier = next_frontier;
        if current_frontier.is_empty() {
            break;
        }
    }

    let result = best_trust[target];
    if result == f64::NEG_INFINITY {
        Ok(0.0)
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- TrustRelation ---

    #[test]
    fn test_trust_relation_valid() {
        let r = TrustRelation::new(0, 1, 0.5).unwrap();
        assert_eq!(r.truster, 0);
        assert_eq!(r.trustee, 1);
        assert!((r.trust_level - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_trust_relation_out_of_range() {
        assert!(TrustRelation::new(0, 1, 1.5).is_err());
        assert!(TrustRelation::new(0, 1, -1.5).is_err());
    }

    #[test]
    fn test_trust_relation_nan() {
        assert!(TrustRelation::new(0, 1, f64::NAN).is_err());
    }

    #[test]
    fn test_trust_relation_boundaries() {
        assert!(TrustRelation::new(0, 1, 1.0).is_ok());
        assert!(TrustRelation::new(0, 1, -1.0).is_ok());
        assert!(TrustRelation::new(0, 1, 0.0).is_ok());
    }

    // --- TrustNetwork ---

    #[test]
    fn test_trust_network_add_and_get() {
        let mut net = TrustNetwork::new(3);
        net.add_trust(0, 1, 0.8).unwrap();
        let level = net.get_trust(0, 1).unwrap();
        assert_eq!(level, Some(0.8));
    }

    #[test]
    fn test_trust_network_get_nonexistent() {
        let net = TrustNetwork::new(3);
        assert_eq!(net.get_trust(0, 1).unwrap(), None);
    }

    #[test]
    fn test_trust_network_out_of_bounds() {
        let mut net = TrustNetwork::new(3);
        assert!(net.add_trust(0, 5, 0.5).is_err());
        assert!(net.get_trust(5, 0).is_err());
    }

    #[test]
    fn test_trust_network_asymmetric() {
        let mut net = TrustNetwork::new(3);
        net.add_trust(0, 1, 0.9).unwrap();
        // Reverse direction has no trust
        assert_eq!(net.get_trust(1, 0).unwrap(), None);
    }

    // --- trust_decay ---

    #[test]
    fn test_trust_decay_zero_time() {
        let d = trust_decay(0.8, 0.0, 0.5).unwrap();
        assert!((d - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_trust_decay_positive() {
        let d = trust_decay(0.8, 10.0, 0.1).unwrap();
        let expected = 0.8 * (-1.0_f64).exp();
        assert!((d - expected).abs() < 1e-10);
    }

    #[test]
    fn test_trust_decay_negative_trust() {
        let d = trust_decay(-0.5, 5.0, 0.2).unwrap();
        assert!(d > -0.5); // decays toward zero
        assert!(d < 0.0); // still negative
    }

    #[test]
    fn test_trust_decay_invalid_rate() {
        assert!(trust_decay(0.5, 1.0, -0.1).is_err());
    }

    // --- betrayal_impact ---

    #[test]
    fn test_betrayal_full_severity() {
        let t = betrayal_impact(0.8, 1.0).unwrap();
        assert!((t - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_betrayal_zero_severity() {
        let t = betrayal_impact(0.8, 0.0).unwrap();
        assert!((t - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_betrayal_half() {
        let t = betrayal_impact(0.8, 0.5).unwrap();
        assert!((t - 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_betrayal_invalid_severity() {
        assert!(betrayal_impact(0.5, 1.5).is_err());
        assert!(betrayal_impact(0.5, -0.1).is_err());
    }

    // --- reputation_aggregate ---

    #[test]
    fn test_reputation_unanimous() {
        let mut net = TrustNetwork::new(4);
        net.add_trust(0, 3, 0.8).unwrap();
        net.add_trust(1, 3, 0.8).unwrap();
        net.add_trust(2, 3, 0.8).unwrap();
        let rep = reputation_aggregate(&net, 3).unwrap();
        assert!((rep.score - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_reputation_mixed() {
        let mut net = TrustNetwork::new(3);
        net.add_trust(0, 2, 0.9).unwrap();
        net.add_trust(1, 2, -0.3).unwrap();
        let rep = reputation_aggregate(&net, 2).unwrap();
        // weighted: (0.9*0.9 + (-0.3)*0.3) / (0.9 + 0.3) = (0.81 - 0.09) / 1.2 = 0.6
        assert!((rep.score - 0.6).abs() < 1e-10);
    }

    #[test]
    fn test_reputation_no_trusters() {
        let net = TrustNetwork::new(3);
        let rep = reputation_aggregate(&net, 0).unwrap();
        assert!((rep.score - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_reputation_out_of_bounds() {
        let net = TrustNetwork::new(3);
        assert!(reputation_aggregate(&net, 5).is_err());
    }

    // --- trust_propagation ---

    #[test]
    fn test_propagation_direct() {
        let mut net = TrustNetwork::new(3);
        net.add_trust(0, 1, 0.8).unwrap();
        let t = trust_propagation(&net, 0, 1, 3, 0.9).unwrap();
        // Direct: 0.8 * 0.9^1 = 0.72
        assert!((t - 0.72).abs() < 1e-10);
    }

    #[test]
    fn test_propagation_two_hops() {
        let mut net = TrustNetwork::new(3);
        net.add_trust(0, 1, 0.8).unwrap();
        net.add_trust(1, 2, 0.8).unwrap();
        let t = trust_propagation(&net, 0, 2, 3, 1.0).unwrap();
        // 0.8 * 0.8 * 1.0^2 = 0.64
        assert!((t - 0.64).abs() < 1e-10);
    }

    #[test]
    fn test_propagation_no_path() {
        let net = TrustNetwork::new(3);
        let t = trust_propagation(&net, 0, 2, 5, 0.9).unwrap();
        assert!((t - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_propagation_hop_limit() {
        let mut net = TrustNetwork::new(4);
        net.add_trust(0, 1, 0.8).unwrap();
        net.add_trust(1, 2, 0.8).unwrap();
        net.add_trust(2, 3, 0.8).unwrap();
        // Path exists at 3 hops but max_hops = 2
        let t = trust_propagation(&net, 0, 3, 2, 0.9).unwrap();
        assert!((t - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_propagation_self() {
        let net = TrustNetwork::new(3);
        let t = trust_propagation(&net, 1, 1, 3, 0.9).unwrap();
        assert!((t - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_propagation_invalid() {
        let net = TrustNetwork::new(3);
        assert!(trust_propagation(&net, 0, 5, 3, 0.9).is_err()); // OOB
        assert!(trust_propagation(&net, 0, 1, 0, 0.9).is_err()); // max_hops=0
        assert!(trust_propagation(&net, 0, 1, 3, 0.0).is_err()); // decay=0
        assert!(trust_propagation(&net, 0, 1, 3, 1.5).is_err()); // decay>1
    }

    // --- serde roundtrips ---

    #[test]
    fn test_trust_relation_serde_roundtrip() {
        let r = TrustRelation::new(0, 1, 0.7).unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let back: TrustRelation = serde_json::from_str(&json).unwrap();
        assert_eq!(r.truster, back.truster);
        assert!((r.trust_level - back.trust_level).abs() < 1e-10);
    }

    #[test]
    fn test_trust_network_serde_roundtrip() {
        let mut net = TrustNetwork::new(3);
        net.add_trust(0, 1, 0.5).unwrap();
        let json = serde_json::to_string(&net).unwrap();
        let back: TrustNetwork = serde_json::from_str(&json).unwrap();
        assert_eq!(net.agent_count, back.agent_count);
    }

    #[test]
    fn test_reputation_score_serde_roundtrip() {
        let r = ReputationScore::new(2, 0.75);
        let json = serde_json::to_string(&r).unwrap();
        let back: ReputationScore = serde_json::from_str(&json).unwrap();
        assert_eq!(r.agent, back.agent);
        assert!((r.score - back.score).abs() < 1e-10);
    }

    #[test]
    fn test_trust_relation_deserialize_rejects_invalid() {
        // trust_level > 1.0 is invalid
        let json = r#"{"truster":0,"trustee":1,"trust_level":2.0}"#;
        let result: core::result::Result<TrustRelation, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
