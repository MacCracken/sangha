# Usage Guide

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sangha = "1"
```

## Module Overview

Sangha is organized as independent modules. Import only what you need:

```rust
use sangha::network;
use sangha::game_theory;
use sangha::trust;
```

## Common Patterns

### Creating and analyzing a network

```rust
use sangha::network;

// Generate a small-world network
let net = network::watts_strogatz(100, 4, 0.1)?;

// Or a scale-free network
let net = network::barabasi_albert_with_seed(100, 3, 42)?;

// Analyze it
let cc = network::average_clustering_coefficient(&net)?;
let apl = network::average_path_length(&net)?;
let d = network::density(&net);
let bc = network::betweenness_centrality(&net, 0)?;
```

### Running a game

```rust
use sangha::game_theory;

let pd = game_theory::prisoners_dilemma();
let equilibria = game_theory::find_nash_equilibria(&pd);

// Iterated game
let (s1, s2) = game_theory::iterated_prisoners_dilemma(
    game_theory::tit_for_tat,
    game_theory::always_defect,
    100,
);
```

### Coalition analysis (for NPC factions)

```rust
use sangha::coalition;

// Define a 3-player majority game
let mut values = vec![0.0; 8];
values[0b011] = 1.0; // {0,1} wins
values[0b101] = 1.0; // {0,2} wins
values[0b110] = 1.0; // {1,2} wins
values[0b111] = 1.0; // {0,1,2} wins
let game = coalition::CoalitionGame::new(3, values)?;

// Compute each player's power
let sv = coalition::shapley_value(&game)?;

// Check if an allocation is stable
let status = coalition::is_core_stable(&game, &[0.4, 0.3, 0.3])?;
```

### Trust and reputation

```rust
use sangha::trust;

let mut net = trust::TrustNetwork::new(4);
net.add_trust(0, 1, 0.9)?;  // Alice trusts Bob
net.add_trust(1, 2, 0.7)?;  // Bob trusts Carol

// Indirect trust: Alice → Carol via Bob
let indirect = trust::trust_propagation(&net, 0, 2, 5, 0.9)?;

// Trust decays without reinforcement
let decayed = trust::trust_decay(0.9, 30.0, 0.05)?;
```

### Emotional contagion on a network

```rust
use sangha::contagion;

let states = vec![
    contagion::EmotionalState::new(0.9, 1.0)?,  // happy, susceptible
    contagion::EmotionalState::new(0.1, 1.0)?,  // sad, susceptible
];
let adj = vec![vec![(1, 1.0)], vec![(0, 1.0)]];
let config = contagion::HatfieldConfig::new(0.5, 0.2)?;

let new_states = contagion::hatfield_contagion_step(&states, &adj, &config, 0.1)?;
```

## Serde Support

All types implement `Serialize` and `Deserialize`. After deserializing, call `validate()` to ensure invariants:

```rust
let game: coalition::CoalitionGame = serde_json::from_str(&json)?;
game.validate()?;  // ensures 2^n entries, finite values, etc.
```

## Error Handling

All fallible operations return `Result<T, SanghaError>`. The error type is `#[non_exhaustive]` with four variants:

- `InvalidNetwork` — graph structure issues
- `InvalidPopulation` — population parameter issues
- `SimulationFailed` — convergence failures
- `ComputationError` — general math/parameter errors
