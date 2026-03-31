# Sangha

**सङ्घ** (Sanskrit: community, assembly) — Sociology engine for social networks, game theory, and group dynamics.

Part of the [AGNOS](https://github.com/MacCracken/agnosticos) science crate ecosystem.

## Key Capabilities

- **Social Networks**: Watts-Strogatz small-world, Barabasi-Albert scale-free, BFS shortest path, average path length, betweenness centrality, clustering coefficient, degree distribution, density
- **Game Theory**: Nash equilibria, prisoner's dilemma, iterated games, tit-for-tat
- **Coordination**: N-player public goods, sealed-bid auctions (first/second price), tragedy of the commons, repeated game discounting, folk theorem
- **Coalition Game Theory**: Shapley value, core stability, coalition value, faction merge/split
- **Collective Decision-Making**: Plurality vote, Borda count, Condorcet winner, majority rule, wisdom of crowds, Condorcet jury theorem
- **Trust & Reputation**: Directed trust networks, trust propagation, reputation aggregation, exponential decay, betrayal impact
- **Emotional Contagion**: Hatfield model with feedback, SIS dynamics, mood propagation, epidemic threshold
- **Opinion Dynamics**: Deffuant bounded confidence, echo chamber detection, consensus
- **Group Dynamics**: Tuckman stages, social loafing (Ringelmann), groupthink risk, collective intelligence
- **Population Models**: Logistic growth, SIR epidemiological model, herd immunity
- **Social Influence**: Conformity (Asch), social proof, Bass diffusion
- **Inequality**: Gini coefficient, Lorenz curve

## Quick Start

```rust
use sangha::{population, inequality, game_theory, coalition, collective, trust};

// Herd immunity threshold for R0 = 3
let h = population::herd_immunity_threshold(3.0).unwrap();
assert!((h - 2.0 / 3.0).abs() < 1e-10); // ~66.7%

// Gini coefficient
let g = inequality::gini_coefficient(&[100.0, 100.0, 100.0]).unwrap();
assert!(g.abs() < 1e-10); // perfect equality

// Nash equilibrium of prisoner's dilemma
let eq = game_theory::find_nash_equilibria(&game_theory::prisoners_dilemma());
assert_eq!(eq[0].player1, game_theory::Strategy::Defect);

// Shapley value of a 3-player majority game
let mut values = vec![0.0; 8];
values[0b011] = 1.0; values[0b101] = 1.0;
values[0b110] = 1.0; values[0b111] = 1.0;
let game = coalition::CoalitionGame::new(3, values).unwrap();
let sv = coalition::shapley_value(&game).unwrap();
assert!((sv.values[0] - 1.0 / 3.0).abs() < 1e-10);

// Condorcet jury theorem: 101 jurors with 60% individual accuracy
let prob = collective::jury_theorem(0.6, 101).unwrap();
assert!(prob > 0.97);
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | Yes | Standard library support |
| `hisab` | No | Advanced math via hisab |
| `pramana` | No | Statistics via pramana |
| `logging` | No | Tracing subscriber |
| `full` | No | All features |

## License

GPL-3.0-only
