//! Integration tests for sangha.

use sangha::coalition;
use sangha::collective;
use sangha::contagion;
use sangha::coordination;
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

#[test]
fn test_borda_elects_broadly_preferred() {
    // Candidate A is first choice of minority but broadly hated;
    // Candidate B is everyone's second choice → Borda should pick B
    let ballots = vec![
        collective::RankedBallot::new(vec![0, 1, 2]),
        collective::RankedBallot::new(vec![2, 1, 0]),
        collective::RankedBallot::new(vec![2, 1, 0]),
    ];
    let result = collective::borda_count(&ballots, 3).unwrap();
    // B (index 1) gets 1+1+1=3 Borda pts, A gets 2+0+0=2, C gets 0+2+2=4
    assert_eq!(result.winner, Some(2));
}

#[test]
fn test_jury_theorem_increases_with_size() {
    let p3 = collective::jury_theorem(0.6, 3).unwrap();
    let p11 = collective::jury_theorem(0.6, 11).unwrap();
    let p101 = collective::jury_theorem(0.6, 101).unwrap();
    assert!(p3 < p11);
    assert!(p11 < p101);
}

#[test]
fn test_shapley_symmetric_players_equal() {
    // 3-player majority game: all symmetric → Shapley = 1/3 each
    let mut values = vec![0.0; 8];
    values[0b011] = 1.0;
    values[0b101] = 1.0;
    values[0b110] = 1.0;
    values[0b111] = 1.0;
    let game = coalition::CoalitionGame::new(3, values).unwrap();
    let sv = coalition::shapley_value(&game).unwrap();
    for &v in &sv.values {
        assert!((v - 1.0 / 3.0).abs() < 1e-10);
    }
}

#[test]
fn test_public_goods_social_dilemma() {
    let game = coordination::PublicGoodsGame::new(4, 2.0, 10.0).unwrap();
    let nash = coordination::free_rider_equilibrium(&game).unwrap();
    let opt = coordination::social_optimum(&game).unwrap();
    let nash_out = coordination::public_goods_round(&game, &nash).unwrap();
    let opt_out = coordination::public_goods_round(&game, &opt).unwrap();
    let nash_total: f64 = nash_out.payoffs.iter().sum();
    let opt_total: f64 = opt_out.payoffs.iter().sum();
    assert!(nash_total < opt_total);
}

#[test]
fn test_hatfield_emotions_converge() {
    let mut states = vec![
        contagion::EmotionalState::new(0.9, 1.0).unwrap(),
        contagion::EmotionalState::new(0.1, 1.0).unwrap(),
    ];
    let adj = vec![vec![(1, 1.0)], vec![(0, 1.0)]];
    let config = contagion::HatfieldConfig::new(0.5, 0.0).unwrap();

    for _ in 0..200 {
        states = contagion::hatfield_contagion_step(&states, &adj, &config, 0.1).unwrap();
    }
    assert!(contagion::emotional_convergence(&states, 0.01).unwrap());
}
