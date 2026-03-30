//! Game theory — Nash equilibria, prisoner's dilemma, evolutionary strategies.

extern crate alloc;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::error::{SanghaError, Result};

/// A 2x2 payoff matrix for two-player games.
///
/// `payoffs[i][j]` gives `(player1_payoff, player2_payoff)` when player 1
/// chooses strategy `i` and player 2 chooses strategy `j`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoffMatrix {
    /// 2x2 matrix of (player1, player2) payoffs.
    pub payoffs: [[(f64, f64); 2]; 2],
}

/// A strategy in a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Strategy {
    /// Cooperate.
    Cooperate,
    /// Defect.
    Defect,
}

/// A Nash equilibrium outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NashEquilibrium {
    /// Player 1's strategy.
    pub player1: Strategy,
    /// Player 2's strategy.
    pub player2: Strategy,
    /// Payoffs at equilibrium.
    pub payoffs: (f64, f64),
}

/// Create the standard prisoner's dilemma payoff matrix.
///
/// |          | Cooperate | Defect |
/// |----------|-----------|--------|
/// | Cooperate | (3, 3)    | (0, 5) |
/// | Defect    | (5, 0)    | (1, 1) |
#[inline]
#[must_use]
pub fn prisoners_dilemma() -> PayoffMatrix {
    PayoffMatrix {
        payoffs: [
            [(3.0, 3.0), (0.0, 5.0)], // row = Cooperate
            [(5.0, 0.0), (1.0, 1.0)], // row = Defect
        ],
    }
}

/// Find all pure-strategy Nash equilibria in a 2x2 game.
///
/// A cell (i, j) is a Nash equilibrium if:
/// - Player 1 cannot improve by switching rows (given column j)
/// - Player 2 cannot improve by switching columns (given row i)
#[must_use = "returns Nash equilibria without side effects"]
pub fn find_nash_equilibria(matrix: &PayoffMatrix) -> Vec<NashEquilibrium> {
    let mut equilibria = Vec::new();
    let strategies = [Strategy::Cooperate, Strategy::Defect];

    for i in 0..2 {
        for j in 0..2 {
            let (p1, p2) = matrix.payoffs[i][j];
            let other_i = 1 - i;
            let other_j = 1 - j;

            // Player 1 best response: given j, is i the best row?
            let p1_alt = matrix.payoffs[other_i][j].0;
            // Player 2 best response: given i, is j the best column?
            let p2_alt = matrix.payoffs[i][other_j].1;

            if p1 >= p1_alt && p2 >= p2_alt {
                equilibria.push(NashEquilibrium {
                    player1: strategies[i],
                    player2: strategies[j],
                    payoffs: (p1, p2),
                });
            }
        }
    }

    equilibria
}

/// Run an iterated prisoner's dilemma between two strategies.
///
/// Returns `(score1, score2)` after `rounds` interactions.
///
/// Strategy functions take the opponent's previous move (None for first round)
/// and return Cooperate or Defect.
#[must_use = "returns the scores without side effects"]
pub fn iterated_prisoners_dilemma<F1, F2>(
    mut strategy1: F1,
    mut strategy2: F2,
    rounds: usize,
) -> (f64, f64)
where
    F1: FnMut(Option<Strategy>) -> Strategy,
    F2: FnMut(Option<Strategy>) -> Strategy,
{
    let pd = prisoners_dilemma();
    let mut score1 = 0.0;
    let mut score2 = 0.0;
    let mut prev1 = None;
    let mut prev2 = None;

    for _ in 0..rounds {
        let move1 = strategy1(prev2);
        let move2 = strategy2(prev1);

        let i = match move1 {
            Strategy::Cooperate => 0,
            Strategy::Defect => 1,
        };
        let j = match move2 {
            Strategy::Cooperate => 0,
            Strategy::Defect => 1,
        };

        let (p1, p2) = pd.payoffs[i][j];
        score1 += p1;
        score2 += p2;

        prev1 = Some(move1);
        prev2 = Some(move2);
    }

    (score1, score2)
}

/// Tit-for-tat strategy: cooperate first, then copy opponent's last move.
#[inline]
#[must_use]
pub fn tit_for_tat(opponent_last: Option<Strategy>) -> Strategy {
    opponent_last.unwrap_or(Strategy::Cooperate)
}

/// Always cooperate strategy.
#[inline]
#[must_use]
pub fn always_cooperate(_opponent_last: Option<Strategy>) -> Strategy {
    Strategy::Cooperate
}

/// Always defect strategy.
#[inline]
#[must_use]
pub fn always_defect(_opponent_last: Option<Strategy>) -> Strategy {
    Strategy::Defect
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prisoners_dilemma_payoffs() {
        let pd = prisoners_dilemma();
        assert_eq!(pd.payoffs[0][0], (3.0, 3.0)); // C,C
        assert_eq!(pd.payoffs[0][1], (0.0, 5.0)); // C,D
        assert_eq!(pd.payoffs[1][0], (5.0, 0.0)); // D,C
        assert_eq!(pd.payoffs[1][1], (1.0, 1.0)); // D,D
    }

    #[test]
    fn test_nash_equilibrium_pd() {
        let pd = prisoners_dilemma();
        let equilibria = find_nash_equilibria(&pd);
        // Prisoner's dilemma has one Nash equilibrium: (Defect, Defect)
        assert_eq!(equilibria.len(), 1);
        assert_eq!(equilibria[0].player1, Strategy::Defect);
        assert_eq!(equilibria[0].player2, Strategy::Defect);
        assert_eq!(equilibria[0].payoffs, (1.0, 1.0));
    }

    #[test]
    fn test_tit_for_tat_first_move() {
        assert_eq!(tit_for_tat(None), Strategy::Cooperate);
    }

    #[test]
    fn test_tit_for_tat_copies() {
        assert_eq!(tit_for_tat(Some(Strategy::Defect)), Strategy::Defect);
        assert_eq!(tit_for_tat(Some(Strategy::Cooperate)), Strategy::Cooperate);
    }

    #[test]
    fn test_iterated_pd_tft_vs_always_cooperate() {
        let (s1, s2) = iterated_prisoners_dilemma(tit_for_tat, always_cooperate, 10);
        // Both cooperate every round: 3 * 10 = 30 each
        assert!((s1 - 30.0).abs() < 1e-10);
        assert!((s2 - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_iterated_pd_always_defect_dominates() {
        let (s1, s2) = iterated_prisoners_dilemma(always_defect, always_cooperate, 10);
        // Defector gets 5 * 10 = 50, cooperator gets 0 * 10 = 0
        assert!((s1 - 50.0).abs() < 1e-10);
        assert!((s2 - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_payoff_matrix_serde_roundtrip() {
        let pd = prisoners_dilemma();
        let json = serde_json::to_string(&pd).unwrap();
        let back: PayoffMatrix = serde_json::from_str(&json).unwrap();
        assert_eq!(pd.payoffs[0][0], back.payoffs[0][0]);
    }

    #[test]
    fn test_strategy_serde_roundtrip() {
        let s = Strategy::Cooperate;
        let json = serde_json::to_string(&s).unwrap();
        let back: Strategy = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn test_nash_equilibrium_serde_roundtrip() {
        let ne = NashEquilibrium {
            player1: Strategy::Defect,
            player2: Strategy::Defect,
            payoffs: (1.0, 1.0),
        };
        let json = serde_json::to_string(&ne).unwrap();
        let back: NashEquilibrium = serde_json::from_str(&json).unwrap();
        assert_eq!(ne.player1, back.player1);
    }
}
