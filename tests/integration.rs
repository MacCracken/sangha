//! Integration tests for sangha.

use sangha::game_theory;
use sangha::inequality;
use sangha::network;
use sangha::population;

#[test]
fn test_watts_strogatz_regular_lattice_clustering() {
    let net = network::watts_strogatz(20, 4, 0.0).unwrap();
    let cc = network::average_clustering_coefficient(&net).unwrap();
    assert!(cc > 0.3);
}

#[test]
fn test_prisoners_dilemma_nash() {
    let pd = game_theory::prisoners_dilemma();
    let eq = game_theory::find_nash_equilibria(&pd);
    assert_eq!(eq.len(), 1);
    assert_eq!(eq[0].player1, game_theory::Strategy::Defect);
    assert_eq!(eq[0].player2, game_theory::Strategy::Defect);
}

#[test]
fn test_gini_equal() {
    let g = inequality::gini_coefficient(&[50.0, 50.0, 50.0, 50.0]).unwrap();
    assert!(g.abs() < 1e-10);
}

#[test]
fn test_herd_immunity_r0_3() {
    let h = population::herd_immunity_threshold(3.0).unwrap();
    assert!((h - 2.0 / 3.0).abs() < 1e-10);
}

#[test]
fn test_sir_r0_below_1_declines() {
    let state = population::sir_step(0.99, 0.01, 0.0, 0.1, 0.5, 0.1).unwrap();
    assert!(state.i < 0.01);
}

#[test]
fn test_logistic_growth_max_at_half_k() {
    let dn_half = population::logistic_growth(500.0, 1.0, 1000.0).unwrap();
    let dn_quarter = population::logistic_growth(250.0, 1.0, 1000.0).unwrap();
    assert!(dn_half > dn_quarter);
}
