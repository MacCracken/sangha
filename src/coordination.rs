//! Multi-agent coordination — public goods games, auctions, mechanism design.

use serde::{Deserialize, Serialize};

use crate::error::{
    Result, SanghaError, validate_finite, validate_non_negative, validate_positive,
};

/// Configuration for an N-player public goods game.
///
/// Each player has an endowment they may contribute to a public pool.
/// The pool is multiplied by `multiplier` and split evenly among all players.
///
/// Deserialization validates invariants automatically.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct PublicGoodsGame {
    /// Number of players.
    pub player_count: usize,
    /// Multiplication factor for the public pool (must be > 1).
    pub multiplier: f64,
    /// Each player's endowment (uniform).
    pub endowment: f64,
}

impl<'de> Deserialize<'de> for PublicGoodsGame {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> core::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            player_count: usize,
            multiplier: f64,
            endowment: f64,
        }
        let raw = Raw::deserialize(deserializer)?;
        PublicGoodsGame::new(raw.player_count, raw.multiplier, raw.endowment)
            .map_err(serde::de::Error::custom)
    }
}

impl PublicGoodsGame {
    /// Create a new public goods game.
    ///
    /// # Errors
    ///
    /// Returns error if `player_count` is 0, `multiplier <= 1`, or `endowment <= 0`.
    pub fn new(player_count: usize, multiplier: f64, endowment: f64) -> Result<Self> {
        if player_count == 0 {
            return Err(SanghaError::ComputationError(
                "player_count must be > 0".into(),
            ));
        }
        validate_finite(multiplier, "multiplier")?;
        if multiplier <= 1.0 {
            return Err(SanghaError::ComputationError(
                "multiplier must be > 1.0 for a social dilemma".into(),
            ));
        }
        validate_positive(endowment, "endowment")?;
        Ok(Self {
            player_count,
            multiplier,
            endowment,
        })
    }

    /// Validate that this game is well-formed.
    ///
    /// Call this after deserialization to ensure invariants hold.
    ///
    /// # Errors
    ///
    /// Returns error if `player_count` is 0, `multiplier <= 1`, or `endowment <= 0`.
    pub fn validate(&self) -> Result<()> {
        if self.player_count == 0 {
            return Err(SanghaError::ComputationError(
                "player_count must be > 0".into(),
            ));
        }
        validate_finite(self.multiplier, "multiplier")?;
        if self.multiplier <= 1.0 {
            return Err(SanghaError::ComputationError(
                "multiplier must be > 1.0 for a social dilemma".into(),
            ));
        }
        validate_positive(self.endowment, "endowment")
    }
}

/// Outcome of a public goods game round.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PublicGoodsOutcome {
    /// Payoff for each player.
    pub payoffs: Vec<f64>,
    /// Total contributed to the public pool.
    pub total_contribution: f64,
}

impl PublicGoodsOutcome {
    /// Create a new public goods outcome.
    #[inline]
    #[must_use]
    pub fn new(payoffs: Vec<f64>, total_contribution: f64) -> Self {
        Self {
            payoffs,
            total_contribution,
        }
    }
}

/// Sealed-bid auction type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AuctionType {
    /// Winner pays their own bid.
    FirstPrice,
    /// Winner pays the second-highest bid (Vickrey auction).
    SecondPrice,
}

/// Result of an auction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AuctionResult {
    /// Index of the winning bidder.
    pub winner: usize,
    /// Price paid by the winner.
    pub price: f64,
}

impl AuctionResult {
    /// Create a new auction result.
    #[inline]
    #[must_use]
    pub fn new(winner: usize, price: f64) -> Self {
        Self { winner, price }
    }
}

/// Play one round of a public goods game.
///
/// Each player contributes `contributions[i]` (between 0 and endowment).
/// The total contribution is multiplied and split evenly.
///
/// `payoff_i = (endowment - contribution_i) + multiplier * total / player_count`
///
/// # Errors
///
/// Returns error if contributions length mismatches, or any contribution is
/// negative or exceeds the endowment.
#[must_use = "returns the game outcome without side effects"]
pub fn public_goods_round(
    game: &PublicGoodsGame,
    contributions: &[f64],
) -> Result<PublicGoodsOutcome> {
    if contributions.len() != game.player_count {
        return Err(SanghaError::ComputationError(format!(
            "contributions length {} != player_count {}",
            contributions.len(),
            game.player_count
        )));
    }

    let n = game.player_count as f64;
    let mut total = 0.0;

    for (i, &c) in contributions.iter().enumerate() {
        validate_non_negative(c, &format!("contributions[{i}]"))?;
        // Use a scaled tolerance: 1e-9 relative to endowment
        if c > game.endowment * (1.0 + 1e-9) + 1e-15 {
            return Err(SanghaError::ComputationError(format!(
                "contributions[{i}] = {c} exceeds endowment {}",
                game.endowment
            )));
        }
        total += c;
    }

    let public_share = game.multiplier * total / n;
    let payoffs: Vec<f64> = contributions
        .iter()
        .map(|&c| (game.endowment - c) + public_share)
        .collect();

    Ok(PublicGoodsOutcome::new(payoffs, total))
}

/// Nash equilibrium contributions (free-rider equilibrium).
///
/// When `multiplier / player_count < 1`, the dominant strategy is to contribute
/// nothing — every dollar contributed returns less than a dollar to the contributor.
///
/// # Errors
///
/// Returns error if the game is invalid.
#[inline]
#[must_use = "returns the equilibrium contributions without side effects"]
pub fn free_rider_equilibrium(game: &PublicGoodsGame) -> Result<Vec<f64>> {
    let mpcr = game.multiplier / game.player_count as f64;
    if mpcr >= 1.0 {
        // When MPCR >= 1, contributing is individually rational
        Ok(vec![game.endowment; game.player_count])
    } else {
        Ok(vec![0.0; game.player_count])
    }
}

/// Socially optimal contributions.
///
/// When `multiplier > 1`, the social optimum is for everyone to contribute
/// their full endowment, since each dollar contributed returns `multiplier`
/// dollars total (though only `multiplier/n` to the contributor).
///
/// # Errors
///
/// Returns error if the game is invalid.
#[inline]
#[must_use = "returns the optimal contributions without side effects"]
pub fn social_optimum(game: &PublicGoodsGame) -> Result<Vec<f64>> {
    // multiplier > 1 is guaranteed by PublicGoodsGame::new
    Ok(vec![game.endowment; game.player_count])
}

/// Run a sealed-bid auction.
///
/// - `FirstPrice`: winner pays their own bid.
/// - `SecondPrice` (Vickrey): winner pays the second-highest bid.
///
/// In case of tied highest bids, the first bidder (lowest index) wins.
///
/// # Errors
///
/// Returns error if `bids` is empty or any bid is negative or non-finite.
#[must_use = "returns the auction result without side effects"]
pub fn sealed_bid_auction(bids: &[f64], auction_type: AuctionType) -> Result<AuctionResult> {
    if bids.is_empty() {
        return Err(SanghaError::ComputationError("no bids provided".into()));
    }
    for (i, &b) in bids.iter().enumerate() {
        validate_non_negative(b, &format!("bids[{i}]"))?;
    }

    // Find winner (highest bidder, first index on tie)
    let mut winner = 0;
    let mut highest = bids[0];
    let mut second_highest = 0.0_f64;

    for (i, &b) in bids.iter().enumerate().skip(1) {
        if b > highest {
            second_highest = highest;
            highest = b;
            winner = i;
        } else if b > second_highest {
            second_highest = b;
        }
    }

    let price = match auction_type {
        AuctionType::FirstPrice => highest,
        AuctionType::SecondPrice => second_highest,
    };

    Ok(AuctionResult::new(winner, price))
}

/// Mechanism efficiency: ratio of actual to optimal social welfare.
///
/// Returns a value in `[0, 1]` when `optimal >= actual >= 0`.
///
/// # Errors
///
/// Returns error if `optimal` is non-positive or values are non-finite.
#[inline]
#[must_use = "returns the efficiency ratio without side effects"]
pub fn mechanism_efficiency(actual_welfare: f64, optimal_welfare: f64) -> Result<f64> {
    validate_finite(actual_welfare, "actual_welfare")?;
    validate_positive(optimal_welfare, "optimal_welfare")?;
    Ok((actual_welfare / optimal_welfare).clamp(0.0, 1.0))
}

/// Configuration for a tragedy of the commons game.
///
/// Each player extracts from a shared resource. Overextraction depletes the resource.
///
/// `π_i = e_i * (1 - E/K) - c * e_i`
///
/// where `e_i` is player i's extraction, `E = Σ e_i`, `K` is capacity, `c` is cost.
///
/// Reference: Hardin (1968), *Science* 162.
///
/// Deserialization validates invariants automatically.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct TragedyOfCommons {
    /// Number of players.
    pub player_count: usize,
    /// Total resource capacity (K > 0).
    pub resource_capacity: f64,
    /// Cost per unit of extraction (c >= 0, c < K).
    pub extraction_cost: f64,
}

impl TragedyOfCommons {
    /// Create a new tragedy of the commons game.
    ///
    /// # Errors
    ///
    /// Returns error if `player_count` is 0, `resource_capacity` is not positive,
    /// `extraction_cost` is negative, or `extraction_cost >= resource_capacity`.
    pub fn new(player_count: usize, resource_capacity: f64, extraction_cost: f64) -> Result<Self> {
        if player_count == 0 {
            return Err(SanghaError::ComputationError(
                "player_count must be > 0".into(),
            ));
        }
        validate_positive(resource_capacity, "resource_capacity")?;
        validate_non_negative(extraction_cost, "extraction_cost")?;
        if extraction_cost >= resource_capacity {
            return Err(SanghaError::ComputationError(
                "extraction_cost must be < resource_capacity".into(),
            ));
        }
        Ok(Self {
            player_count,
            resource_capacity,
            extraction_cost,
        })
    }

    /// Validate after deserialization.
    ///
    /// # Errors
    ///
    /// Returns error if the game parameters are invalid.
    pub fn validate(&self) -> Result<()> {
        if self.player_count == 0 {
            return Err(SanghaError::ComputationError(
                "player_count must be > 0".into(),
            ));
        }
        validate_positive(self.resource_capacity, "resource_capacity")?;
        validate_non_negative(self.extraction_cost, "extraction_cost")?;
        if self.extraction_cost >= self.resource_capacity {
            return Err(SanghaError::ComputationError(
                "extraction_cost must be < resource_capacity".into(),
            ));
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for TragedyOfCommons {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> core::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            player_count: usize,
            resource_capacity: f64,
            extraction_cost: f64,
        }
        let raw = Raw::deserialize(deserializer)?;
        TragedyOfCommons::new(raw.player_count, raw.resource_capacity, raw.extraction_cost)
            .map_err(serde::de::Error::custom)
    }
}

/// Compute payoffs for one round of the tragedy of the commons.
///
/// `π_i = e_i * (1 - E/K) - c * e_i`
///
/// # Errors
///
/// Returns error if `extractions` length != `player_count`, or any extraction
/// is negative or non-finite.
#[must_use = "returns the payoffs without side effects"]
pub fn tragedy_of_commons_round(game: &TragedyOfCommons, extractions: &[f64]) -> Result<Vec<f64>> {
    if extractions.len() != game.player_count {
        return Err(SanghaError::ComputationError(format!(
            "extractions length {} != player_count {}",
            extractions.len(),
            game.player_count
        )));
    }

    let mut total = 0.0;
    for (i, &e) in extractions.iter().enumerate() {
        validate_non_negative(e, &format!("extractions[{i}]"))?;
        total += e;
    }

    let k = game.resource_capacity;
    let c = game.extraction_cost;
    let payoffs = extractions
        .iter()
        .map(|&e| e * (1.0 - total / k) - c * e)
        .collect();

    Ok(payoffs)
}

/// Cournot-Nash equilibrium extraction levels for the tragedy of the commons.
///
/// Each player extracts: `e* = (K - c) / (n + 1)`
///
/// Total extraction: `E* = n * (K - c) / (n + 1)`, which exceeds the social optimum.
#[inline]
#[must_use = "returns the equilibrium extractions without side effects"]
pub fn commons_nash_equilibrium(game: &TragedyOfCommons) -> Result<Vec<f64>> {
    let e_star = (game.resource_capacity - game.extraction_cost) / (game.player_count as f64 + 1.0);
    Ok(vec![e_star; game.player_count])
}

/// Socially optimal extraction levels for the tragedy of the commons.
///
/// Total optimal extraction: `E_opt = (K - c) / 2`, split equally.
/// Per player: `e_opt = (K - c) / (2n)`.
#[inline]
#[must_use = "returns the optimal extractions without side effects"]
pub fn commons_social_optimum(game: &TragedyOfCommons) -> Result<Vec<f64>> {
    let e_opt = (game.resource_capacity - game.extraction_cost) / (2.0 * game.player_count as f64);
    Ok(vec![e_opt; game.player_count])
}

/// Discounted sum of a constant payoff over a finite number of rounds.
///
/// `V = Σ_{t=0}^{rounds-1} payoff * δ^t = payoff * (1 - δ^rounds) / (1 - δ)`
///
/// For `δ = 1.0`, returns `payoff * rounds`.
///
/// # Errors
///
/// Returns error if `discount_factor` not in \[0, 1\], `payoff` not finite, or `rounds` is 0.
#[inline]
#[must_use = "returns the discounted sum without side effects"]
pub fn repeated_game_discount(payoff: f64, rounds: usize, discount_factor: f64) -> Result<f64> {
    validate_finite(payoff, "payoff")?;
    validate_finite(discount_factor, "discount_factor")?;
    if !(0.0..=1.0).contains(&discount_factor) {
        return Err(SanghaError::ComputationError(format!(
            "discount_factor must be in [0, 1], got {discount_factor}"
        )));
    }
    if rounds == 0 {
        return Err(SanghaError::ComputationError("rounds must be > 0".into()));
    }

    if (discount_factor - 1.0).abs() < f64::EPSILON {
        Ok(payoff * rounds as f64)
    } else if discount_factor.abs() < f64::EPSILON {
        Ok(payoff) // only the first round counts
    } else {
        Ok(payoff * (1.0 - discount_factor.powi(rounds as i32)) / (1.0 - discount_factor))
    }
}

/// Check if cooperation is sustainable under the folk theorem.
///
/// Cooperation is sustainable in an infinitely repeated game if:
/// `δ >= (T - R) / (T - P)`
///
/// where `T` = temptation, `R` = reward (mutual cooperation),
/// `P` = punishment (mutual defection), `δ` = discount factor.
///
/// Requires `T > R > P` (prisoner's dilemma ordering).
///
/// # Errors
///
/// Returns error if `T <= R`, `R <= P`, `discount` not in \[0, 1\],
/// or any value is non-finite.
///
/// Reference: Friedman (1971), *Review of Economic Studies* 38(1).
#[inline]
#[must_use = "returns whether cooperation is sustainable without side effects"]
pub fn folk_theorem_threshold(
    temptation: f64,
    reward: f64,
    punishment: f64,
    discount: f64,
) -> Result<bool> {
    validate_finite(temptation, "temptation")?;
    validate_finite(reward, "reward")?;
    validate_finite(punishment, "punishment")?;
    validate_finite(discount, "discount")?;
    if temptation <= reward {
        return Err(SanghaError::ComputationError(
            "temptation must be > reward".into(),
        ));
    }
    if reward <= punishment {
        return Err(SanghaError::ComputationError(
            "reward must be > punishment".into(),
        ));
    }
    if !(0.0..=1.0).contains(&discount) {
        return Err(SanghaError::ComputationError(format!(
            "discount must be in [0, 1], got {discount}"
        )));
    }

    let threshold = (temptation - reward) / (temptation - punishment);
    Ok(discount >= threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- PublicGoodsGame ---

    #[test]
    fn test_public_goods_all_contribute() {
        let game = PublicGoodsGame::new(4, 2.0, 10.0).unwrap();
        let contributions = vec![10.0, 10.0, 10.0, 10.0];
        let outcome = public_goods_round(&game, &contributions).unwrap();
        // Total = 40, pool = 2*40 = 80, share = 80/4 = 20
        // Payoff = (10-10) + 20 = 20
        for &p in &outcome.payoffs {
            assert!((p - 20.0).abs() < 1e-10);
        }
        assert!((outcome.total_contribution - 40.0).abs() < 1e-10);
    }

    #[test]
    fn test_public_goods_free_rider() {
        let game = PublicGoodsGame::new(4, 2.0, 10.0).unwrap();
        // Player 0 free-rides, others contribute fully
        let contributions = vec![0.0, 10.0, 10.0, 10.0];
        let outcome = public_goods_round(&game, &contributions).unwrap();
        // Total = 30, pool = 60, share = 15
        // Player 0: (10-0) + 15 = 25 (highest!)
        // Others: (10-10) + 15 = 15
        assert!((outcome.payoffs[0] - 25.0).abs() < 1e-10);
        assert!((outcome.payoffs[1] - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_public_goods_none_contribute() {
        let game = PublicGoodsGame::new(3, 2.0, 10.0).unwrap();
        let contributions = vec![0.0, 0.0, 0.0];
        let outcome = public_goods_round(&game, &contributions).unwrap();
        // Everyone keeps endowment
        for &p in &outcome.payoffs {
            assert!((p - 10.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_public_goods_wrong_length_error() {
        let game = PublicGoodsGame::new(3, 2.0, 10.0).unwrap();
        assert!(public_goods_round(&game, &[5.0, 5.0]).is_err());
    }

    #[test]
    fn test_public_goods_negative_contribution_error() {
        let game = PublicGoodsGame::new(3, 2.0, 10.0).unwrap();
        assert!(public_goods_round(&game, &[-1.0, 5.0, 5.0]).is_err());
    }

    #[test]
    fn test_public_goods_excess_contribution_error() {
        let game = PublicGoodsGame::new(3, 2.0, 10.0).unwrap();
        assert!(public_goods_round(&game, &[11.0, 5.0, 5.0]).is_err());
    }

    #[test]
    fn test_game_invalid_multiplier() {
        assert!(PublicGoodsGame::new(3, 0.5, 10.0).is_err());
        assert!(PublicGoodsGame::new(3, 1.0, 10.0).is_err());
    }

    #[test]
    fn test_game_zero_players() {
        assert!(PublicGoodsGame::new(0, 2.0, 10.0).is_err());
    }

    // --- free_rider_equilibrium / social_optimum ---

    #[test]
    fn test_free_rider_equilibrium_low_mpcr() {
        // MPCR = 2/4 = 0.5 < 1 → contribute nothing
        let game = PublicGoodsGame::new(4, 2.0, 10.0).unwrap();
        let eq = free_rider_equilibrium(&game).unwrap();
        for &c in &eq {
            assert!((c - 0.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_free_rider_equilibrium_high_mpcr() {
        // MPCR = 5/3 ≈ 1.67 >= 1 → contribute fully
        let game = PublicGoodsGame::new(3, 5.0, 10.0).unwrap();
        let eq = free_rider_equilibrium(&game).unwrap();
        for &c in &eq {
            assert!((c - 10.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_social_optimum_full_contribution() {
        let game = PublicGoodsGame::new(4, 2.0, 10.0).unwrap();
        let opt = social_optimum(&game).unwrap();
        for &c in &opt {
            assert!((c - 10.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_social_dilemma_gap() {
        // The gap between Nash and social optimum demonstrates the social dilemma
        let game = PublicGoodsGame::new(4, 2.0, 10.0).unwrap();
        let nash = free_rider_equilibrium(&game).unwrap();
        let opt = social_optimum(&game).unwrap();
        let nash_outcome = public_goods_round(&game, &nash).unwrap();
        let opt_outcome = public_goods_round(&game, &opt).unwrap();
        let nash_welfare: f64 = nash_outcome.payoffs.iter().sum();
        let opt_welfare: f64 = opt_outcome.payoffs.iter().sum();
        assert!(nash_welfare < opt_welfare); // social dilemma!
    }

    // --- sealed_bid_auction ---

    #[test]
    fn test_first_price_auction() {
        let result = sealed_bid_auction(&[10.0, 30.0, 20.0], AuctionType::FirstPrice).unwrap();
        assert_eq!(result.winner, 1);
        assert!((result.price - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_second_price_auction() {
        let result = sealed_bid_auction(&[10.0, 30.0, 20.0], AuctionType::SecondPrice).unwrap();
        assert_eq!(result.winner, 1);
        assert!((result.price - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_auction_single_bidder() {
        let result = sealed_bid_auction(&[100.0], AuctionType::SecondPrice).unwrap();
        assert_eq!(result.winner, 0);
        assert!((result.price - 0.0).abs() < 1e-10); // no second bid
    }

    #[test]
    fn test_auction_tied_bids() {
        // First bidder wins on tie
        let result = sealed_bid_auction(&[50.0, 50.0], AuctionType::FirstPrice).unwrap();
        assert_eq!(result.winner, 0);
    }

    #[test]
    fn test_auction_empty_error() {
        assert!(sealed_bid_auction(&[], AuctionType::FirstPrice).is_err());
    }

    #[test]
    fn test_auction_negative_bid_error() {
        assert!(sealed_bid_auction(&[10.0, -5.0], AuctionType::FirstPrice).is_err());
    }

    // --- mechanism_efficiency ---

    #[test]
    fn test_mechanism_efficiency_perfect() {
        let e = mechanism_efficiency(100.0, 100.0).unwrap();
        assert!((e - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_mechanism_efficiency_half() {
        let e = mechanism_efficiency(50.0, 100.0).unwrap();
        assert!((e - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_mechanism_efficiency_zero_optimal_error() {
        assert!(mechanism_efficiency(50.0, 0.0).is_err());
    }

    // --- serde roundtrips ---

    #[test]
    fn test_public_goods_game_serde_roundtrip() {
        let game = PublicGoodsGame::new(3, 2.0, 10.0).unwrap();
        let json = serde_json::to_string(&game).unwrap();
        let back: PublicGoodsGame = serde_json::from_str(&json).unwrap();
        assert_eq!(game.player_count, back.player_count);
    }

    #[test]
    fn test_public_goods_outcome_serde_roundtrip() {
        let outcome = PublicGoodsOutcome::new(vec![15.0, 15.0], 20.0);
        let json = serde_json::to_string(&outcome).unwrap();
        let back: PublicGoodsOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(outcome.payoffs, back.payoffs);
    }

    #[test]
    fn test_auction_type_serde_roundtrip() {
        let at = AuctionType::SecondPrice;
        let json = serde_json::to_string(&at).unwrap();
        let back: AuctionType = serde_json::from_str(&json).unwrap();
        assert_eq!(at, back);
    }

    #[test]
    fn test_auction_result_serde_roundtrip() {
        let ar = AuctionResult::new(2, 50.0);
        let json = serde_json::to_string(&ar).unwrap();
        let back: AuctionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(ar.winner, back.winner);
    }

    // --- audit tests ---

    #[test]
    fn test_mechanism_efficiency_clamp_negative() {
        let e = mechanism_efficiency(-10.0, 100.0).unwrap();
        assert!((e - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_mechanism_efficiency_clamp_above_one() {
        let e = mechanism_efficiency(200.0, 100.0).unwrap();
        assert!((e - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_free_rider_equilibrium_mpcr_exactly_one() {
        // MPCR = 3/3 = 1.0 exactly → should contribute fully
        let game = PublicGoodsGame::new(3, 3.0, 10.0).unwrap();
        let eq = free_rider_equilibrium(&game).unwrap();
        for &c in &eq {
            assert!((c - 10.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_social_optimum_direct() {
        let game = PublicGoodsGame::new(5, 1.5, 20.0).unwrap();
        let opt = social_optimum(&game).unwrap();
        assert_eq!(opt.len(), 5);
        for &c in &opt {
            assert!((c - 20.0).abs() < 1e-10);
        }
    }

    // --- TragedyOfCommons ---

    #[test]
    fn test_tragedy_new_valid() {
        let g = TragedyOfCommons::new(5, 1000.0, 10.0);
        assert!(g.is_ok());
    }

    #[test]
    fn test_tragedy_new_invalid() {
        assert!(TragedyOfCommons::new(0, 1000.0, 10.0).is_err()); // no players
        assert!(TragedyOfCommons::new(5, -1.0, 10.0).is_err()); // negative capacity
        assert!(TragedyOfCommons::new(5, 1000.0, 1000.0).is_err()); // cost >= capacity
    }

    #[test]
    fn test_tragedy_round_basic() {
        let game = TragedyOfCommons::new(2, 100.0, 5.0).unwrap();
        // Player 0 extracts 10, player 1 extracts 20. Total = 30.
        // π_0 = 10*(1-30/100) - 5*10 = 10*0.7 - 50 = 7 - 50 = -43
        // π_1 = 20*(1-30/100) - 5*20 = 20*0.7 - 100 = 14 - 100 = -86
        let payoffs = tragedy_of_commons_round(&game, &[10.0, 20.0]).unwrap();
        assert!((payoffs[0] - (-43.0)).abs() < 1e-10);
        assert!((payoffs[1] - (-86.0)).abs() < 1e-10);
    }

    #[test]
    fn test_tragedy_round_zero_extraction() {
        let game = TragedyOfCommons::new(3, 100.0, 5.0).unwrap();
        let payoffs = tragedy_of_commons_round(&game, &[0.0, 0.0, 0.0]).unwrap();
        for &p in &payoffs {
            assert!((p - 0.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_tragedy_round_overextraction() {
        let game = TragedyOfCommons::new(2, 100.0, 0.0).unwrap();
        let payoffs = tragedy_of_commons_round(&game, &[80.0, 80.0]).unwrap();
        // Total=160 > K=100, so (1-E/K) = -0.6, payoffs are negative
        for &p in &payoffs {
            assert!(p < 0.0);
        }
    }

    #[test]
    fn test_tragedy_round_wrong_length() {
        let game = TragedyOfCommons::new(3, 100.0, 5.0).unwrap();
        assert!(tragedy_of_commons_round(&game, &[10.0, 20.0]).is_err());
    }

    #[test]
    fn test_commons_nash_formula() {
        // e* = (K - c) / (n + 1) = (1000 - 10) / 6 = 165.0
        let game = TragedyOfCommons::new(5, 1000.0, 10.0).unwrap();
        let nash = commons_nash_equilibrium(&game).unwrap();
        assert_eq!(nash.len(), 5);
        for &e in &nash {
            assert!((e - 165.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_commons_social_optimum_formula() {
        // e_opt = (K - c) / (2n) = (1000 - 10) / 10 = 99.0
        let game = TragedyOfCommons::new(5, 1000.0, 10.0).unwrap();
        let opt = commons_social_optimum(&game).unwrap();
        for &e in &opt {
            assert!((e - 99.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_commons_nash_exceeds_optimum() {
        let game = TragedyOfCommons::new(5, 1000.0, 10.0).unwrap();
        let nash = commons_nash_equilibrium(&game).unwrap();
        let opt = commons_social_optimum(&game).unwrap();
        let nash_total: f64 = nash.iter().sum();
        let opt_total: f64 = opt.iter().sum();
        assert!(nash_total > opt_total); // the tragedy
    }

    #[test]
    fn test_commons_nash_vs_optimum_payoffs() {
        let game = TragedyOfCommons::new(5, 1000.0, 10.0).unwrap();
        let nash = commons_nash_equilibrium(&game).unwrap();
        let opt = commons_social_optimum(&game).unwrap();
        let nash_payoffs = tragedy_of_commons_round(&game, &nash).unwrap();
        let opt_payoffs = tragedy_of_commons_round(&game, &opt).unwrap();
        let nash_welfare: f64 = nash_payoffs.iter().sum();
        let opt_welfare: f64 = opt_payoffs.iter().sum();
        assert!(nash_welfare < opt_welfare); // Nash is worse for society
    }

    // --- repeated_game_discount ---

    #[test]
    fn test_repeated_discount_single_round() {
        let v = repeated_game_discount(10.0, 1, 0.9).unwrap();
        assert!((v - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_repeated_discount_geometric() {
        // V = 10 * (1 - 0.9^5) / (1 - 0.9) = 10 * (1 - 0.59049) / 0.1 = 40.951
        let v = repeated_game_discount(10.0, 5, 0.9).unwrap();
        let expected = 10.0 * (1.0 - 0.9_f64.powi(5)) / 0.1;
        assert!((v - expected).abs() < 1e-8);
    }

    #[test]
    fn test_repeated_discount_delta_one() {
        let v = repeated_game_discount(10.0, 100, 1.0).unwrap();
        assert!((v - 1000.0).abs() < 1e-10);
    }

    #[test]
    fn test_repeated_discount_delta_zero() {
        let v = repeated_game_discount(10.0, 100, 0.0).unwrap();
        assert!((v - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_repeated_discount_invalid() {
        assert!(repeated_game_discount(10.0, 0, 0.9).is_err()); // rounds=0
        assert!(repeated_game_discount(10.0, 5, 1.5).is_err()); // delta>1
        assert!(repeated_game_discount(10.0, 5, -0.1).is_err()); // delta<0
    }

    // --- folk_theorem_threshold ---

    #[test]
    fn test_folk_theorem_cooperate() {
        // PD: T=5, R=3, P=1. Threshold = (5-3)/(5-1) = 0.5
        assert!(folk_theorem_threshold(5.0, 3.0, 1.0, 0.9).unwrap());
    }

    #[test]
    fn test_folk_theorem_defect() {
        assert!(!folk_theorem_threshold(5.0, 3.0, 1.0, 0.3).unwrap());
    }

    #[test]
    fn test_folk_theorem_boundary() {
        // At exactly the threshold: δ = 0.5 >= 0.5 → true
        assert!(folk_theorem_threshold(5.0, 3.0, 1.0, 0.5).unwrap());
    }

    #[test]
    fn test_folk_theorem_invalid_ordering() {
        assert!(folk_theorem_threshold(3.0, 5.0, 1.0, 0.9).is_err()); // T <= R
        assert!(folk_theorem_threshold(5.0, 1.0, 3.0, 0.9).is_err()); // R <= P
    }

    #[test]
    fn test_folk_theorem_pd_payoffs() {
        // Standard PD: T=5, R=3, P=1. Threshold = 2/4 = 0.5
        let threshold_met = folk_theorem_threshold(5.0, 3.0, 1.0, 0.5).unwrap();
        assert!(threshold_met);
        let below = folk_theorem_threshold(5.0, 3.0, 1.0, 0.49).unwrap();
        assert!(!below);
    }

    // --- serde roundtrips ---

    #[test]
    fn test_tragedy_serde_roundtrip() {
        let game = TragedyOfCommons::new(5, 1000.0, 10.0).unwrap();
        let json = serde_json::to_string(&game).unwrap();
        let back: TragedyOfCommons = serde_json::from_str(&json).unwrap();
        assert_eq!(game.player_count, back.player_count);
    }

    #[test]
    fn test_public_goods_deserialize_rejects_invalid() {
        // multiplier <= 1 is invalid
        let json = r#"{"player_count":3,"multiplier":0.5,"endowment":10.0}"#;
        let result: core::result::Result<PublicGoodsGame, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_tragedy_deserialize_rejects_invalid() {
        // extraction_cost >= resource_capacity is invalid
        let json = r#"{"player_count":3,"resource_capacity":100.0,"extraction_cost":200.0}"#;
        let result: core::result::Result<TragedyOfCommons, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
