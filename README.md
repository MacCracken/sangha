# Sangha

**सङ्घ** (Sanskrit: community, assembly) — Sociology engine for social networks, game theory, and group dynamics.

Part of the [AGNOS](https://github.com/MacCracken/agnosticos) science crate ecosystem.

## Key Capabilities

- **Social Networks**: Watts-Strogatz small-world, clustering coefficient, degree distribution
- **Game Theory**: Nash equilibria, prisoner's dilemma, iterated games, tit-for-tat
- **Opinion Dynamics**: Deffuant bounded confidence, echo chamber detection, consensus
- **Group Dynamics**: Tuckman stages, social loafing (Ringelmann), groupthink risk
- **Population Models**: Logistic growth, SIR epidemiological model, herd immunity
- **Social Influence**: Conformity (Asch), social proof, Bass diffusion
- **Inequality**: Gini coefficient, Lorenz curve

## Quick Start

```rust
use sangha::{population, inequality, game_theory};

// Herd immunity threshold for R0 = 3
let h = population::herd_immunity_threshold(3.0).unwrap();
assert!((h - 2.0 / 3.0).abs() < 1e-10); // ~66.7%

// Gini coefficient
let g = inequality::gini_coefficient(&[100.0, 100.0, 100.0]).unwrap();
assert!(g.abs() < 1e-10); // perfect equality

// Nash equilibrium of prisoner's dilemma
let eq = game_theory::find_nash_equilibria(&game_theory::prisoners_dilemma());
assert_eq!(eq[0].player1, game_theory::Strategy::Defect);
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
