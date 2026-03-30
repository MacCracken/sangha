//! Multi-agent coordination — public goods games, auctions, mechanism design.

use serde::{Deserialize, Serialize};

use crate::error::{
    Result, SanghaError, validate_finite, validate_non_negative, validate_positive,
};

/// Configuration for an N-player public goods game.
///
/// Each player has an endowment they may contribute to a public pool.
/// The pool is multiplied by `multiplier` and split evenly among all players.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PublicGoodsGame {
    /// Number of players.
    pub player_count: usize,
    /// Multiplication factor for the public pool (must be > 1).
    pub multiplier: f64,
    /// Each player's endowment (uniform).
    pub endowment: f64,
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
        if c > game.endowment + f64::EPSILON {
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
}
