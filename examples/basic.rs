//! Basic examples of sangha sociology engine capabilities.

fn main() {
    // === Network Analysis ===
    // Generate a Barabasi-Albert scale-free network
    let net = sangha::network::barabasi_albert_with_seed(100, 3, 42).unwrap();
    let avg_path = sangha::network::average_path_length(&net).unwrap();
    let d = sangha::network::density(&net);
    println!("BA network (100 nodes): density={d:.3}, avg path={avg_path:.2}");

    // Watts-Strogatz small-world
    let ws = sangha::network::watts_strogatz(100, 4, 0.1).unwrap();
    let cc = sangha::network::average_clustering_coefficient(&ws).unwrap();
    println!("WS network: clustering={cc:.3}");

    // === Game Theory ===
    let pd = sangha::game_theory::prisoners_dilemma();
    let eq = sangha::game_theory::find_nash_equilibria(&pd);
    println!("\nPrisoner's dilemma Nash equilibria: {}", eq.len());

    // Tit-for-tat vs always-defect over 100 rounds
    let (s1, s2) = sangha::game_theory::iterated_prisoners_dilemma(
        sangha::game_theory::tit_for_tat,
        sangha::game_theory::always_defect,
        100,
    );
    println!("TFT vs Always-Defect (100 rounds): {s1:.0} vs {s2:.0}");

    // === Coordination ===
    // Public goods game — the social dilemma
    let pgg = sangha::coordination::PublicGoodsGame::new(10, 2.0, 10.0).unwrap();
    let nash = sangha::coordination::free_rider_equilibrium(&pgg).unwrap();
    let opt = sangha::coordination::social_optimum(&pgg).unwrap();
    println!(
        "\nPublic goods: Nash contribution={:.1}, optimal={:.1}",
        nash[0], opt[0]
    );

    // Tragedy of the commons
    let commons = sangha::coordination::TragedyOfCommons::new(10, 1000.0, 5.0).unwrap();
    let nash_e = sangha::coordination::commons_nash_equilibrium(&commons).unwrap();
    let opt_e = sangha::coordination::commons_social_optimum(&commons).unwrap();
    println!(
        "Commons: Nash extraction={:.1}/player, optimal={:.1}/player",
        nash_e[0], opt_e[0]
    );

    // Folk theorem: can cooperation survive?
    let coop = sangha::coordination::folk_theorem_threshold(5.0, 3.0, 1.0, 0.9).unwrap();
    println!("Folk theorem (delta=0.9): cooperation sustainable = {coop}");

    // === Coalition Game Theory ===
    // 3-player majority game: Shapley values
    let mut values = vec![0.0; 8];
    values[0b011] = 1.0;
    values[0b101] = 1.0;
    values[0b110] = 1.0;
    values[0b111] = 1.0;
    let game = sangha::coalition::CoalitionGame::new(3, values).unwrap();
    let sv = sangha::coalition::shapley_value(&game).unwrap();
    println!("\nShapley values (3-player majority): {:?}", sv.values);

    // === Collective Decision-Making ===
    let ballots = vec![
        sangha::collective::RankedBallot::new(vec![0, 1, 2]),
        sangha::collective::RankedBallot::new(vec![1, 0, 2]),
        sangha::collective::RankedBallot::new(vec![0, 2, 1]),
    ];
    let borda = sangha::collective::borda_count(&ballots, 3).unwrap();
    println!(
        "\nBorda count winner: {:?}, scores: {:?}",
        borda.winner, borda.scores
    );

    // Condorcet jury theorem
    let prob = sangha::collective::jury_theorem(0.6, 101).unwrap();
    println!("Jury theorem (p=0.6, n=101): P(correct)={prob:.4}");

    // === Trust & Reputation ===
    let mut trust_net = sangha::trust::TrustNetwork::new(4);
    trust_net.add_trust(0, 1, 0.9).unwrap();
    trust_net.add_trust(1, 2, 0.8).unwrap();
    trust_net.add_trust(2, 3, 0.7).unwrap();
    let indirect = sangha::trust::trust_propagation(&trust_net, 0, 3, 5, 0.9).unwrap();
    println!("\nTrust 0->3 via chain: {indirect:.4}");

    let decayed = sangha::trust::trust_decay(0.9, 30.0, 0.05).unwrap();
    println!("Trust after 30 time steps (decay=0.05): {decayed:.4}");

    let post_betrayal = sangha::trust::betrayal_impact(0.9, 0.7).unwrap();
    println!("Trust after betrayal (severity=0.7): {post_betrayal:.4}");

    // === Contagion ===
    let states = vec![
        sangha::contagion::EmotionalState::new(0.9, 1.0).unwrap(),
        sangha::contagion::EmotionalState::new(0.1, 1.0).unwrap(),
    ];
    let adj = vec![vec![(1, 1.0)], vec![(0, 1.0)]];
    let config = sangha::contagion::HatfieldConfig::new(0.5, 0.2).unwrap();
    let after = sangha::contagion::hatfield_contagion_step(&states, &adj, &config, 0.1).unwrap();
    println!(
        "\nHatfield step: [{:.3}, {:.3}] -> [{:.3}, {:.3}]",
        states[0].valence, states[1].valence, after[0].valence, after[1].valence
    );

    // === Population & Epidemiology ===
    let r0 = sangha::population::r_naught(0.5, 0.2).unwrap();
    let h = sangha::population::herd_immunity_threshold(r0).unwrap();
    println!("\nR0={r0:.1}, herd immunity threshold={:.1}%", h * 100.0);

    // === Inequality ===
    let incomes = vec![20.0, 30.0, 50.0, 100.0, 200.0];
    let g = sangha::inequality::gini_coefficient(&incomes).unwrap();
    println!("\nGini coefficient: {g:.3}");
}
