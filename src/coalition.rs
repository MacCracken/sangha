//! Coalition game theory — Shapley value, core stability, faction dynamics.

use serde::{Deserialize, Serialize};

use crate::error::{Result, SanghaError, validate_finite};

/// A coalition game defined by a characteristic function.
///
/// Maps subsets of `player_count` players to coalition values via bitmask indexing:
/// `values[mask]` gives `v(S)` where each bit in `mask` indicates player membership.
///
/// Deserialization validates invariants automatically.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct CoalitionGame {
    /// Number of players (max 20 for practical bitmask limits).
    pub player_count: usize,
    /// Coalition values indexed by bitmask. Length must be `2^player_count`.
    pub values: Vec<f64>,
}

impl<'de> Deserialize<'de> for CoalitionGame {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> core::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            player_count: usize,
            values: Vec<f64>,
        }
        let raw = Raw::deserialize(deserializer)?;
        CoalitionGame::new(raw.player_count, raw.values).map_err(serde::de::Error::custom)
    }
}

impl CoalitionGame {
    /// Create a new coalition game.
    ///
    /// # Errors
    ///
    /// Returns error if `player_count > 20`, `values.len() != 2^player_count`,
    /// or any value is non-finite.
    pub fn new(player_count: usize, values: Vec<f64>) -> Result<Self> {
        if player_count > 20 {
            return Err(SanghaError::ComputationError(
                "player_count must be <= 20 (bitmask limit)".into(),
            ));
        }
        let expected_len = 1 << player_count;
        if values.len() != expected_len {
            return Err(SanghaError::ComputationError(format!(
                "values length {} != 2^{player_count} = {expected_len}",
                values.len()
            )));
        }
        for (i, &v) in values.iter().enumerate() {
            validate_finite(v, &format!("values[{i}]"))?;
        }
        Ok(Self {
            player_count,
            values,
        })
    }

    /// Validate that this game is well-formed.
    ///
    /// Call this after deserialization to ensure invariants hold.
    ///
    /// # Errors
    ///
    /// Returns error if `player_count > 20`, `values.len() != 2^player_count`,
    /// or any value is non-finite.
    pub fn validate(&self) -> Result<()> {
        if self.player_count > 20 {
            return Err(SanghaError::ComputationError(
                "player_count must be <= 20 (bitmask limit)".into(),
            ));
        }
        let expected_len = 1 << self.player_count;
        if self.values.len() != expected_len {
            return Err(SanghaError::ComputationError(format!(
                "values length {} != 2^{} = {expected_len}",
                self.values.len(),
                self.player_count
            )));
        }
        for (i, &v) in self.values.iter().enumerate() {
            validate_finite(v, &format!("values[{i}]"))?;
        }
        Ok(())
    }
}

/// Shapley value allocation for each player.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ShapleyValues {
    /// Shapley value for each player (index = player).
    pub values: Vec<f64>,
}

impl ShapleyValues {
    /// Create a new Shapley values allocation.
    #[inline]
    #[must_use]
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}

/// Coalition stability status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StabilityStatus {
    /// The allocation is in the core: no coalition can improve by deviating.
    Stable,
    /// Some coalition can improve by deviating from the allocation.
    Unstable,
}

/// A coalition structure: partition of players into groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CoalitionStructure {
    /// Each inner `Vec` is a coalition (set of player indices).
    pub coalitions: Vec<Vec<usize>>,
}

impl CoalitionStructure {
    /// Create a new coalition structure.
    #[inline]
    #[must_use]
    pub fn new(coalitions: Vec<Vec<usize>>) -> Self {
        Self { coalitions }
    }
}

/// Compute the Shapley value for each player.
///
/// The Shapley value of player `i` is the expected marginal contribution of `i`
/// across all possible orderings of players:
///
/// `φ_i = Σ_{S ⊆ N\{i}} [|S|!(n-|S|-1)! / n!] · [v(S ∪ {i}) - v(S)]`
///
/// **Complexity**: O(n · 2^n) where n = `player_count`. Practical for n ≤ 20.
///
/// # Errors
///
/// Returns error if the game is invalid.
#[must_use = "returns the Shapley values without side effects"]
pub fn shapley_value(game: &CoalitionGame) -> Result<ShapleyValues> {
    let n = game.player_count;
    if n == 0 {
        return Ok(ShapleyValues::new(vec![]));
    }

    // Pre-compute factorials
    let mut factorial = vec![1.0_f64; n + 1];
    for i in 1..=n {
        factorial[i] = factorial[i - 1] * i as f64;
    }
    let n_fact = factorial[n];

    let mut phi = vec![0.0; n];

    // Iterate over all coalitions S (as bitmasks not containing player i)
    for (i, phi_i) in phi.iter_mut().enumerate() {
        let i_bit = 1usize << i;
        // Iterate over all subsets S of N\{i}
        let mask_without_i = ((1usize << n) - 1) ^ i_bit;
        let mut s = 0usize;
        loop {
            let s_size = s.count_ones() as usize;
            let weight = factorial[s_size] * factorial[n - s_size - 1] / n_fact;
            let marginal = game.values[s | i_bit] - game.values[s];
            *phi_i += weight * marginal;

            if s == mask_without_i {
                break;
            }
            // Enumerate subsets of mask_without_i using Gosper's hack
            s = (s.wrapping_sub(mask_without_i)) & mask_without_i;
        }
    }

    Ok(ShapleyValues::new(phi))
}

/// Look up the value of a specific coalition.
///
/// # Errors
///
/// Returns error if any member index is out of bounds.
#[inline]
#[must_use = "returns the coalition value without side effects"]
pub fn coalition_value(game: &CoalitionGame, members: &[usize]) -> Result<f64> {
    let mut mask = 0usize;
    for &m in members {
        if m >= game.player_count {
            return Err(SanghaError::ComputationError(format!(
                "player index {m} out of bounds for {} players",
                game.player_count
            )));
        }
        mask |= 1 << m;
    }
    Ok(game.values[mask])
}

/// Check if an allocation is in the core (stable).
///
/// An allocation is in the core if:
/// 1. **Efficiency**: `Σ allocation_i = v(N)` (grand coalition value)
/// 2. **Coalition rationality**: for every coalition S, `Σ_{i∈S} allocation_i >= v(S)`
///
/// # Errors
///
/// Returns error if `allocation.len() != game.player_count` or values are non-finite.
#[must_use = "returns the stability status without side effects"]
pub fn is_core_stable(game: &CoalitionGame, allocation: &[f64]) -> Result<StabilityStatus> {
    let n = game.player_count;
    if allocation.len() != n {
        return Err(SanghaError::ComputationError(format!(
            "allocation length {} != player_count {n}",
            allocation.len()
        )));
    }
    for (i, &a) in allocation.iter().enumerate() {
        validate_finite(a, &format!("allocation[{i}]"))?;
    }

    // Check efficiency: sum of allocations = v(N)
    let grand_mask = (1usize << n) - 1;
    let grand_value = game.values[grand_mask];
    let alloc_sum: f64 = allocation.iter().sum();
    if (alloc_sum - grand_value).abs() > 1e-9 {
        return Ok(StabilityStatus::Unstable);
    }

    // Check coalition rationality for all non-empty coalitions
    for mask in 1..=grand_mask {
        let coalition_alloc: f64 = (0..n)
            .filter(|&i| mask & (1 << i) != 0)
            .map(|i| allocation[i])
            .sum();
        if coalition_alloc < game.values[mask] - 1e-9 {
            return Ok(StabilityStatus::Unstable);
        }
    }

    Ok(StabilityStatus::Stable)
}

/// Merge two coalitions in a coalition structure.
///
/// # Errors
///
/// Returns error if `i` or `j` are out of bounds or equal.
#[must_use = "returns the new coalition structure without side effects"]
pub fn merge_coalitions(
    structure: &CoalitionStructure,
    i: usize,
    j: usize,
) -> Result<CoalitionStructure> {
    if i >= structure.coalitions.len() || j >= structure.coalitions.len() {
        return Err(SanghaError::ComputationError(
            "coalition index out of bounds".into(),
        ));
    }
    if i == j {
        return Err(SanghaError::ComputationError(
            "cannot merge a coalition with itself".into(),
        ));
    }

    let (lo, hi) = if i < j { (i, j) } else { (j, i) };
    let mut new_coalitions: Vec<Vec<usize>> = structure.coalitions.clone();
    let removed = new_coalitions.remove(hi);
    new_coalitions[lo].extend(removed);
    Ok(CoalitionStructure::new(new_coalitions))
}

/// Split a coalition at a given boundary index.
///
/// Members `[0..split_point]` stay, members `[split_point..]` form a new coalition.
///
/// # Errors
///
/// Returns error if `coalition_idx` is out of bounds or `split_point` is at a boundary.
#[must_use = "returns the new coalition structure without side effects"]
pub fn split_coalition(
    structure: &CoalitionStructure,
    coalition_idx: usize,
    split_point: usize,
) -> Result<CoalitionStructure> {
    if coalition_idx >= structure.coalitions.len() {
        return Err(SanghaError::ComputationError(
            "coalition index out of bounds".into(),
        ));
    }
    let coalition = &structure.coalitions[coalition_idx];
    if split_point == 0 || split_point >= coalition.len() {
        return Err(SanghaError::ComputationError(
            "split_point must be in (0, coalition.len())".into(),
        ));
    }

    let mut new_coalitions = structure.coalitions.clone();
    let right = new_coalitions[coalition_idx].split_off(split_point);
    new_coalitions.push(right);
    Ok(CoalitionStructure::new(new_coalitions))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: 3-player majority game
    // v(S) = 1 if |S| >= 2, else 0
    fn majority_game_3() -> CoalitionGame {
        let mut values = vec![0.0; 8]; // 2^3
        // Coalitions with 2+ players
        values[0b011] = 1.0; // {0,1}
        values[0b101] = 1.0; // {0,2}
        values[0b110] = 1.0; // {1,2}
        values[0b111] = 1.0; // {0,1,2}
        CoalitionGame::new(3, values).unwrap()
    }

    #[test]
    fn test_shapley_majority_game() {
        let game = majority_game_3();
        let sv = shapley_value(&game).unwrap();
        // All players symmetric → each gets 1/3
        for &v in &sv.values {
            assert!((v - 1.0 / 3.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_shapley_efficiency() {
        // Shapley values sum to v(N)
        let game = majority_game_3();
        let sv = shapley_value(&game).unwrap();
        let sum: f64 = sv.values.iter().sum();
        let grand = game.values[(1 << game.player_count) - 1];
        assert!((sum - grand).abs() < 1e-10);
    }

    #[test]
    fn test_shapley_single_player() {
        let game = CoalitionGame::new(1, vec![0.0, 5.0]).unwrap();
        let sv = shapley_value(&game).unwrap();
        assert!((sv.values[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_shapley_empty_game() {
        let game = CoalitionGame::new(0, vec![0.0]).unwrap();
        let sv = shapley_value(&game).unwrap();
        assert!(sv.values.is_empty());
    }

    #[test]
    fn test_coalition_value_lookup() {
        let game = majority_game_3();
        let v = coalition_value(&game, &[0, 1]).unwrap();
        assert!((v - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_coalition_value_empty() {
        let game = majority_game_3();
        let v = coalition_value(&game, &[]).unwrap();
        assert!((v - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_coalition_value_out_of_bounds() {
        let game = majority_game_3();
        assert!(coalition_value(&game, &[5]).is_err());
    }

    #[test]
    fn test_core_stable_equal_split() {
        let game = majority_game_3();
        // Equal split of v(N)=1: each gets 1/3
        let alloc = vec![1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
        let status = is_core_stable(&game, &alloc).unwrap();
        // In 3-player majority game, any pair gets 1.0 but equal split gives 2/3 to a pair
        // 2/3 < 1.0 → unstable (not in core)
        assert_eq!(status, StabilityStatus::Unstable);
    }

    #[test]
    fn test_core_stable_superadditive() {
        // Simple game where grand coalition = sum of singletons (additive)
        // v({0})=1, v({1})=2, v({2})=3, all pairs/grand = sum of members
        let mut values = vec![0.0; 8];
        values[0b001] = 1.0;
        values[0b010] = 2.0;
        values[0b100] = 3.0;
        values[0b011] = 3.0;
        values[0b101] = 4.0;
        values[0b110] = 5.0;
        values[0b111] = 6.0;
        let game = CoalitionGame::new(3, values).unwrap();
        let alloc = vec![1.0, 2.0, 3.0];
        let status = is_core_stable(&game, &alloc).unwrap();
        assert_eq!(status, StabilityStatus::Stable);
    }

    #[test]
    fn test_core_stable_wrong_total() {
        let game = majority_game_3();
        // Allocation doesn't sum to v(N) → unstable
        let alloc = vec![0.5, 0.5, 0.5]; // sums to 1.5, v(N)=1
        let status = is_core_stable(&game, &alloc).unwrap();
        assert_eq!(status, StabilityStatus::Unstable);
    }

    #[test]
    fn test_core_stable_wrong_length() {
        let game = majority_game_3();
        assert!(is_core_stable(&game, &[0.5, 0.5]).is_err());
    }

    #[test]
    fn test_merge_coalitions() {
        let cs = CoalitionStructure::new(vec![vec![0, 1], vec![2, 3], vec![4]]);
        let merged = merge_coalitions(&cs, 0, 2).unwrap();
        assert_eq!(merged.coalitions.len(), 2);
        assert_eq!(merged.coalitions[0], vec![0, 1, 4]);
        assert_eq!(merged.coalitions[1], vec![2, 3]);
    }

    #[test]
    fn test_merge_coalitions_same_index_error() {
        let cs = CoalitionStructure::new(vec![vec![0], vec![1]]);
        assert!(merge_coalitions(&cs, 0, 0).is_err());
    }

    #[test]
    fn test_merge_coalitions_out_of_bounds() {
        let cs = CoalitionStructure::new(vec![vec![0]]);
        assert!(merge_coalitions(&cs, 0, 5).is_err());
    }

    #[test]
    fn test_split_coalition() {
        let cs = CoalitionStructure::new(vec![vec![0, 1, 2, 3]]);
        let split = split_coalition(&cs, 0, 2).unwrap();
        assert_eq!(split.coalitions.len(), 2);
        assert_eq!(split.coalitions[0], vec![0, 1]);
        assert_eq!(split.coalitions[1], vec![2, 3]);
    }

    #[test]
    fn test_split_coalition_boundary_error() {
        let cs = CoalitionStructure::new(vec![vec![0, 1, 2]]);
        assert!(split_coalition(&cs, 0, 0).is_err()); // can't split at 0
        assert!(split_coalition(&cs, 0, 3).is_err()); // can't split at len
    }

    #[test]
    fn test_game_too_many_players() {
        assert!(CoalitionGame::new(21, vec![0.0; 1 << 21]).is_err());
    }

    #[test]
    fn test_game_wrong_values_length() {
        assert!(CoalitionGame::new(3, vec![0.0; 5]).is_err());
    }

    // --- serde roundtrips ---

    #[test]
    fn test_coalition_game_serde_roundtrip() {
        let game = majority_game_3();
        let json = serde_json::to_string(&game).unwrap();
        let back: CoalitionGame = serde_json::from_str(&json).unwrap();
        assert_eq!(game.player_count, back.player_count);
        assert_eq!(game.values, back.values);
    }

    #[test]
    fn test_shapley_values_serde_roundtrip() {
        let sv = ShapleyValues::new(vec![1.0, 2.0, 3.0]);
        let json = serde_json::to_string(&sv).unwrap();
        let back: ShapleyValues = serde_json::from_str(&json).unwrap();
        assert_eq!(sv.values, back.values);
    }

    #[test]
    fn test_stability_status_serde_roundtrip() {
        let s = StabilityStatus::Stable;
        let json = serde_json::to_string(&s).unwrap();
        let back: StabilityStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn test_coalition_structure_serde_roundtrip() {
        let cs = CoalitionStructure::new(vec![vec![0, 1], vec![2]]);
        let json = serde_json::to_string(&cs).unwrap();
        let back: CoalitionStructure = serde_json::from_str(&json).unwrap();
        assert_eq!(cs.coalitions, back.coalitions);
    }

    // --- audit tests ---

    #[test]
    fn test_shapley_asymmetric_dictator() {
        // Dictator game: player 0 has all power
        // v(S) = 1 if 0 ∈ S, else 0
        let mut values = vec![0.0; 8];
        values[0b001] = 1.0; // {0}
        values[0b011] = 1.0; // {0,1}
        values[0b101] = 1.0; // {0,2}
        values[0b111] = 1.0; // {0,1,2}
        let game = CoalitionGame::new(3, values).unwrap();
        let sv = shapley_value(&game).unwrap();
        assert!((sv.values[0] - 1.0).abs() < 1e-10); // dictator gets all
        assert!((sv.values[1] - 0.0).abs() < 1e-10);
        assert!((sv.values[2] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_core_stable_nan_allocation_error() {
        let game = majority_game_3();
        assert!(is_core_stable(&game, &[f64::NAN, 0.0, 0.0]).is_err());
    }

    #[test]
    fn test_game_nan_values_error() {
        assert!(CoalitionGame::new(1, vec![0.0, f64::NAN]).is_err());
    }

    #[test]
    fn test_coalition_game_deserialize_rejects_invalid() {
        // Wrong values length: player_count=2 needs 4 values, not 2
        let json = r#"{"player_count":2,"values":[0.0,1.0]}"#;
        let result: core::result::Result<CoalitionGame, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
