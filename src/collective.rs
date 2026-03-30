//! Group decision-making — voting methods, wisdom of crowds, jury theorem.

use serde::{Deserialize, Serialize};

use crate::error::{Result, SanghaError, validate_finite};

/// Validate that a ballot is a valid permutation of `0..candidate_count`.
fn validate_ballot(ballot: &RankedBallot, candidate_count: usize) -> Result<()> {
    if ballot.ranking.len() != candidate_count {
        return Err(SanghaError::ComputationError(format!(
            "ballot length {} != candidate_count {candidate_count}",
            ballot.ranking.len()
        )));
    }
    let mut seen = vec![false; candidate_count];
    for &c in &ballot.ranking {
        if c >= candidate_count {
            return Err(SanghaError::ComputationError(format!(
                "candidate index {c} out of bounds for {candidate_count} candidates"
            )));
        }
        if seen[c] {
            return Err(SanghaError::ComputationError(format!(
                "duplicate candidate index {c} in ballot"
            )));
        }
        seen[c] = true;
    }
    Ok(())
}

/// A ranked ballot: candidate indices ordered from most to least preferred.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RankedBallot {
    /// Candidate indices from most to least preferred.
    pub ranking: Vec<usize>,
}

impl RankedBallot {
    /// Create a new ranked ballot.
    #[inline]
    #[must_use]
    pub fn new(ranking: Vec<usize>) -> Self {
        Self { ranking }
    }
}

/// Result of a voting procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VoteResult {
    /// Winning candidate index (`None` if tied).
    pub winner: Option<usize>,
    /// Score for each candidate (index = candidate).
    pub scores: Vec<f64>,
}

impl VoteResult {
    /// Create a new vote result.
    #[inline]
    #[must_use]
    pub fn new(winner: Option<usize>, scores: Vec<f64>) -> Self {
        Self { winner, scores }
    }
}

/// Aggregation method for wisdom-of-crowds estimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AggregationMethod {
    /// Arithmetic mean.
    Mean,
    /// Median.
    Median,
    /// Trimmed mean (remove top and bottom 10%; falls back to plain mean for < 10 estimates).
    TrimmedMean,
}

/// Plurality vote (first-past-the-post).
///
/// Each element of `votes` is the index of the voter's preferred candidate.
/// The candidate with the most votes wins.
///
/// # Errors
///
/// Returns error if `votes` is empty or any vote index >= `candidate_count`.
#[must_use = "returns the vote result without side effects"]
pub fn plurality_vote(votes: &[usize], candidate_count: usize) -> Result<VoteResult> {
    if votes.is_empty() {
        return Err(SanghaError::ComputationError("no votes cast".into()));
    }
    if candidate_count == 0 {
        return Err(SanghaError::ComputationError(
            "candidate_count must be > 0".into(),
        ));
    }

    let mut scores = vec![0.0; candidate_count];
    for &v in votes {
        if v >= candidate_count {
            return Err(SanghaError::ComputationError(format!(
                "vote index {v} out of bounds for {candidate_count} candidates"
            )));
        }
        scores[v] += 1.0;
    }

    let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let winners: Vec<usize> = scores
        .iter()
        .enumerate()
        .filter(|&(_, &s)| (s - max_score).abs() < f64::EPSILON)
        .map(|(i, _)| i)
        .collect();

    let winner = if winners.len() == 1 {
        Some(winners[0])
    } else {
        None // tie
    };

    Ok(VoteResult { winner, scores })
}

/// Borda count: positional voting method.
///
/// Each ballot awards `n-1` points to the first-ranked candidate,
/// `n-2` to the second, and so on down to 0 for the last-ranked.
///
/// # Errors
///
/// Returns error if `ballots` is empty, any ballot is not a valid permutation
/// of `0..candidate_count`, or `candidate_count` is 0.
#[must_use = "returns the vote result without side effects"]
pub fn borda_count(ballots: &[RankedBallot], candidate_count: usize) -> Result<VoteResult> {
    if ballots.is_empty() {
        return Err(SanghaError::ComputationError("no ballots cast".into()));
    }
    if candidate_count == 0 {
        return Err(SanghaError::ComputationError(
            "candidate_count must be > 0".into(),
        ));
    }

    let mut scores = vec![0.0; candidate_count];
    let n = candidate_count as f64;

    for ballot in ballots {
        validate_ballot(ballot, candidate_count)?;
        for (rank, &candidate) in ballot.ranking.iter().enumerate() {
            scores[candidate] += n - 1.0 - rank as f64;
        }
    }

    let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let winners: Vec<usize> = scores
        .iter()
        .enumerate()
        .filter(|&(_, &s)| (s - max_score).abs() < f64::EPSILON)
        .map(|(i, _)| i)
        .collect();

    let winner = if winners.len() == 1 {
        Some(winners[0])
    } else {
        None
    };

    Ok(VoteResult { winner, scores })
}

/// Find the Condorcet winner: the candidate who beats every other
/// candidate in pairwise majority comparison.
///
/// Returns `None` if no Condorcet winner exists (e.g., a Condorcet cycle).
///
/// # Errors
///
/// Returns error if `ballots` is empty or any ballot is malformed.
#[must_use = "returns the Condorcet winner without side effects"]
pub fn condorcet_winner(ballots: &[RankedBallot], candidate_count: usize) -> Result<Option<usize>> {
    if ballots.is_empty() {
        return Err(SanghaError::ComputationError("no ballots cast".into()));
    }
    if candidate_count == 0 {
        return Err(SanghaError::ComputationError(
            "candidate_count must be > 0".into(),
        ));
    }
    if candidate_count == 1 {
        return Ok(Some(0));
    }

    // Build pairwise comparison matrix: pairwise[i][j] = number of voters
    // who prefer candidate i over candidate j
    let n = candidate_count;
    let mut pairwise = vec![vec![0usize; n]; n];

    for ballot in ballots {
        validate_ballot(ballot, n)?;
        // For each pair (a, b) where a appears before b in ranking, a is preferred
        for (pos_a, &a) in ballot.ranking.iter().enumerate() {
            for &b in &ballot.ranking[pos_a + 1..] {
                pairwise[a][b] += 1;
            }
        }
    }

    let total_voters = ballots.len();
    // A Condorcet winner beats all others: pairwise[w][j] > n_voters / 2
    for (w, row) in pairwise.iter().enumerate() {
        let beats_all = (0..n)
            .filter(|&j| j != w)
            .all(|j| row[j] * 2 > total_voters);
        if beats_all {
            return Ok(Some(w));
        }
    }

    Ok(None)
}

/// Simple majority rule: returns `true` if more than half of votes are `true`.
///
/// # Errors
///
/// Returns error if `votes` is empty.
#[inline]
#[must_use = "returns the majority decision without side effects"]
pub fn majority_rule(votes: &[bool]) -> Result<bool> {
    if votes.is_empty() {
        return Err(SanghaError::ComputationError("no votes cast".into()));
    }
    let yes_count = votes.iter().filter(|&&v| v).count();
    Ok(yes_count * 2 > votes.len())
}

/// Wisdom of crowds: aggregate independent estimates using a chosen method.
///
/// - `Mean`: arithmetic mean
/// - `Median`: middle value (or average of two middle values)
/// - `TrimmedMean`: remove top and bottom 10%, then take the mean
///
/// # Errors
///
/// Returns error if `estimates` is empty or contains non-finite values.
#[must_use = "returns the aggregate estimate without side effects"]
pub fn wisdom_of_crowds(estimates: &[f64], method: AggregationMethod) -> Result<f64> {
    if estimates.is_empty() {
        return Err(SanghaError::ComputationError(
            "no estimates provided".into(),
        ));
    }
    for (i, &e) in estimates.iter().enumerate() {
        validate_finite(e, &format!("estimates[{i}]"))?;
    }

    match method {
        AggregationMethod::Mean => {
            let sum: f64 = estimates.iter().sum();
            Ok(sum / estimates.len() as f64)
        }
        AggregationMethod::Median => {
            let mut sorted: Vec<f64> = estimates.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let n = sorted.len();
            if n % 2 == 1 {
                Ok(sorted[n / 2])
            } else {
                Ok((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0)
            }
        }
        AggregationMethod::TrimmedMean => {
            let mut sorted: Vec<f64> = estimates.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let n = sorted.len();
            let trim = (n as f64 * 0.1).floor() as usize;
            // When n < 10, trim is 0 and we fall back to the plain mean
            let trimmed = &sorted[trim..n - trim];
            let sum: f64 = trimmed.iter().sum();
            Ok(sum / trimmed.len() as f64)
        }
    }
}

/// Condorcet jury theorem: probability that a majority vote is correct.
///
/// Given `jury_size` independent jurors each with probability `p` of being
/// correct (where `p > 0.5`), returns the probability that a simple majority
/// reaches the correct decision.
///
/// `P = Σ_{k=⌈n/2⌉}^{n} C(n,k) · p^k · (1-p)^{n-k}`
///
/// # Errors
///
/// Returns error if `p` is not in `(0.5, 1.0)` or `jury_size` is 0.
#[must_use = "returns the majority-correct probability without side effects"]
pub fn jury_theorem(p: f64, jury_size: usize) -> Result<f64> {
    validate_finite(p, "p")?;
    if p <= 0.5 || p >= 1.0 {
        return Err(SanghaError::ComputationError(
            "p must be in (0.5, 1.0) for the jury theorem to hold".into(),
        ));
    }
    if jury_size == 0 {
        return Err(SanghaError::ComputationError(
            "jury_size must be > 0".into(),
        ));
    }

    let n = jury_size;
    let threshold = n / 2 + 1; // majority threshold

    // Use log-space to avoid overflow for large juries
    // ln(C(n,k)) = ln(n!) - ln(k!) - ln((n-k)!)
    // Pre-compute ln factorials
    let mut ln_fact = vec![0.0_f64; n + 1];
    for i in 1..=n {
        ln_fact[i] = ln_fact[i - 1] + (i as f64).ln();
    }

    let ln_p = p.ln();
    let ln_q = (1.0 - p).ln();

    let mut prob = 0.0;
    for k in threshold..=n {
        let ln_binom = ln_fact[n] - ln_fact[k] - ln_fact[n - k];
        let ln_term = ln_binom + k as f64 * ln_p + (n - k) as f64 * ln_q;
        prob += ln_term.exp();
    }

    Ok(prob.min(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- plurality_vote ---

    #[test]
    fn test_plurality_clear_winner() {
        let result = plurality_vote(&[0, 0, 1, 0, 2], 3).unwrap();
        assert_eq!(result.winner, Some(0));
        assert!((result.scores[0] - 3.0).abs() < 1e-10);
        assert!((result.scores[1] - 1.0).abs() < 1e-10);
        assert!((result.scores[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_plurality_tie() {
        let result = plurality_vote(&[0, 1, 0, 1], 2).unwrap();
        assert_eq!(result.winner, None);
    }

    #[test]
    fn test_plurality_single_voter() {
        let result = plurality_vote(&[2], 3).unwrap();
        assert_eq!(result.winner, Some(2));
    }

    #[test]
    fn test_plurality_empty_error() {
        assert!(plurality_vote(&[], 3).is_err());
    }

    #[test]
    fn test_plurality_out_of_bounds_error() {
        assert!(plurality_vote(&[5], 3).is_err());
    }

    #[test]
    fn test_plurality_zero_candidates_error() {
        assert!(plurality_vote(&[0], 0).is_err());
    }

    // --- borda_count ---

    #[test]
    fn test_borda_known_result() {
        // 3 candidates: A=0, B=1, C=2
        // Voter 1: A > B > C → A gets 2, B gets 1, C gets 0
        // Voter 2: B > A > C → B gets 2, A gets 1, C gets 0
        // Voter 3: A > C > B → A gets 2, C gets 1, B gets 0
        // Totals: A=5, B=3, C=1
        let ballots = vec![
            RankedBallot::new(vec![0, 1, 2]),
            RankedBallot::new(vec![1, 0, 2]),
            RankedBallot::new(vec![0, 2, 1]),
        ];
        let result = borda_count(&ballots, 3).unwrap();
        assert_eq!(result.winner, Some(0));
        assert!((result.scores[0] - 5.0).abs() < 1e-10);
        assert!((result.scores[1] - 3.0).abs() < 1e-10);
        assert!((result.scores[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_borda_empty_error() {
        assert!(borda_count(&[], 3).is_err());
    }

    #[test]
    fn test_borda_wrong_ballot_length_error() {
        let ballots = vec![RankedBallot::new(vec![0, 1])]; // 2 candidates but expect 3
        assert!(borda_count(&ballots, 3).is_err());
    }

    // --- condorcet_winner ---

    #[test]
    fn test_condorcet_clear_winner() {
        // A beats B (2-1), A beats C (2-1) → A is Condorcet winner
        let ballots = vec![
            RankedBallot::new(vec![0, 1, 2]),
            RankedBallot::new(vec![0, 2, 1]),
            RankedBallot::new(vec![1, 2, 0]),
        ];
        let winner = condorcet_winner(&ballots, 3).unwrap();
        assert_eq!(winner, Some(0));
    }

    #[test]
    fn test_condorcet_cycle_no_winner() {
        // A>B, B>C, C>A → Condorcet cycle
        let ballots = vec![
            RankedBallot::new(vec![0, 1, 2]),
            RankedBallot::new(vec![1, 2, 0]),
            RankedBallot::new(vec![2, 0, 1]),
        ];
        let winner = condorcet_winner(&ballots, 3).unwrap();
        assert_eq!(winner, None);
    }

    #[test]
    fn test_condorcet_single_candidate() {
        let winner = condorcet_winner(&[RankedBallot::new(vec![0])], 1).unwrap();
        assert_eq!(winner, Some(0));
    }

    // --- majority_rule ---

    #[test]
    fn test_majority_true() {
        assert!(majority_rule(&[true, true, false]).unwrap());
    }

    #[test]
    fn test_majority_false() {
        assert!(!majority_rule(&[true, false, false]).unwrap());
    }

    #[test]
    fn test_majority_even_split_false() {
        // 50-50 is not a majority (need >50%)
        assert!(!majority_rule(&[true, false]).unwrap());
    }

    #[test]
    fn test_majority_empty_error() {
        assert!(majority_rule(&[]).is_err());
    }

    // --- wisdom_of_crowds ---

    #[test]
    fn test_wisdom_mean() {
        let result = wisdom_of_crowds(&[10.0, 20.0, 30.0], AggregationMethod::Mean).unwrap();
        assert!((result - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_wisdom_median_odd() {
        let result = wisdom_of_crowds(&[10.0, 30.0, 20.0], AggregationMethod::Median).unwrap();
        assert!((result - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_wisdom_median_even() {
        let result =
            wisdom_of_crowds(&[10.0, 20.0, 30.0, 40.0], AggregationMethod::Median).unwrap();
        assert!((result - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_wisdom_trimmed_mean() {
        // 20 values: trim 2 from each end
        let mut estimates: Vec<f64> = (1..=20).map(|x| x as f64).collect();
        estimates[0] = 1000.0; // outlier
        estimates[19] = -1000.0; // outlier
        let trimmed = wisdom_of_crowds(&estimates, AggregationMethod::TrimmedMean).unwrap();
        // After trimming outliers, should be close to mean of 2..=19
        let expected: f64 = (3..=18).map(|x| x as f64).sum::<f64>() / 16.0;
        assert!((trimmed - expected).abs() < 1e-10);
    }

    #[test]
    fn test_wisdom_empty_error() {
        assert!(wisdom_of_crowds(&[], AggregationMethod::Mean).is_err());
    }

    #[test]
    fn test_wisdom_nan_error() {
        assert!(wisdom_of_crowds(&[1.0, f64::NAN], AggregationMethod::Mean).is_err());
    }

    // --- jury_theorem ---

    #[test]
    fn test_jury_theorem_high_accuracy() {
        // With p=0.9 and 101 jurors, probability should be very close to 1
        let prob = jury_theorem(0.9, 101).unwrap();
        assert!(prob > 0.999);
    }

    #[test]
    fn test_jury_theorem_single_juror() {
        // With 1 juror, probability = p
        let prob = jury_theorem(0.7, 1).unwrap();
        assert!((prob - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_jury_theorem_three_jurors() {
        // P(majority correct) = C(3,2)*p^2*(1-p) + C(3,3)*p^3
        // = 3*0.6^2*0.4 + 0.6^3 = 0.432 + 0.216 = 0.648
        let prob = jury_theorem(0.6, 3).unwrap();
        assert!((prob - 0.648).abs() < 1e-10);
    }

    #[test]
    fn test_jury_theorem_p_too_low_error() {
        assert!(jury_theorem(0.5, 10).is_err());
        assert!(jury_theorem(0.3, 10).is_err());
    }

    #[test]
    fn test_jury_theorem_p_one_error() {
        assert!(jury_theorem(1.0, 10).is_err());
    }

    #[test]
    fn test_jury_theorem_zero_jurors_error() {
        assert!(jury_theorem(0.7, 0).is_err());
    }

    // --- serde roundtrips ---

    #[test]
    fn test_ranked_ballot_serde_roundtrip() {
        let ballot = RankedBallot::new(vec![2, 0, 1]);
        let json = serde_json::to_string(&ballot).unwrap();
        let back: RankedBallot = serde_json::from_str(&json).unwrap();
        assert_eq!(ballot.ranking, back.ranking);
    }

    #[test]
    fn test_vote_result_serde_roundtrip() {
        let result = VoteResult::new(Some(1), vec![3.0, 5.0, 2.0]);
        let json = serde_json::to_string(&result).unwrap();
        let back: VoteResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.winner, back.winner);
        assert_eq!(result.scores, back.scores);
    }

    #[test]
    fn test_aggregation_method_serde_roundtrip() {
        let method = AggregationMethod::TrimmedMean;
        let json = serde_json::to_string(&method).unwrap();
        let back: AggregationMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(method, back);
    }

    // --- audit tests ---

    #[test]
    fn test_borda_duplicate_candidate_error() {
        let ballots = vec![RankedBallot::new(vec![0, 0, 1])];
        assert!(borda_count(&ballots, 3).is_err());
    }

    #[test]
    fn test_condorcet_duplicate_candidate_error() {
        let ballots = vec![RankedBallot::new(vec![0, 0, 1])];
        assert!(condorcet_winner(&ballots, 3).is_err());
    }

    #[test]
    fn test_borda_tie() {
        // Two candidates, two voters, opposite preferences → tie
        let ballots = vec![RankedBallot::new(vec![0, 1]), RankedBallot::new(vec![1, 0])];
        let result = borda_count(&ballots, 2).unwrap();
        assert_eq!(result.winner, None);
    }

    #[test]
    fn test_condorcet_zero_candidates_error() {
        assert!(condorcet_winner(&[RankedBallot::new(vec![])], 0).is_err());
    }

    #[test]
    fn test_jury_theorem_even_jury() {
        // 4 jurors, p=0.7: threshold = 3 (majority of 4)
        // P = C(4,3)*0.7^3*0.3 + C(4,4)*0.7^4
        //   = 4*0.343*0.3 + 0.2401 = 0.4116 + 0.2401 = 0.6517
        let prob = jury_theorem(0.7, 4).unwrap();
        assert!((prob - 0.6517).abs() < 1e-4);
    }

    #[test]
    fn test_wisdom_trimmed_mean_small_input() {
        // 3 estimates: trim = floor(0.3) = 0, should be plain mean
        let result = wisdom_of_crowds(&[10.0, 20.0, 30.0], AggregationMethod::TrimmedMean).unwrap();
        assert!((result - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_wisdom_single_estimate() {
        let result = wisdom_of_crowds(&[42.0], AggregationMethod::Mean).unwrap();
        assert!((result - 42.0).abs() < 1e-10);
        let result = wisdom_of_crowds(&[42.0], AggregationMethod::Median).unwrap();
        assert!((result - 42.0).abs() < 1e-10);
    }
}
