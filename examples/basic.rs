//! Basic examples of sangha sociology formulas.

fn main() {
    // Gini coefficient
    let incomes = vec![20.0, 30.0, 50.0, 100.0, 200.0];
    let g = sangha::inequality::gini_coefficient(&incomes).unwrap();
    println!("Gini coefficient: {g:.3}");

    // SIR model
    let r0 = sangha::population::r_naught(0.5, 0.2).unwrap();
    let h = sangha::population::herd_immunity_threshold(r0).unwrap();
    println!("R0 = {r0:.1}, herd immunity threshold = {h:.1%}");

    // Prisoner's dilemma
    let pd = sangha::game_theory::prisoners_dilemma();
    let eq = sangha::game_theory::find_nash_equilibria(&pd);
    println!("Nash equilibria: {}", eq.len());

    // Watts-Strogatz network
    let net = sangha::network::watts_strogatz(100, 4, 0.1).unwrap();
    let cc = sangha::network::average_clustering_coefficient(&net).unwrap();
    println!("Small-world clustering: {cc:.3}");
}
