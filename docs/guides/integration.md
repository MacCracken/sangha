# Integration Guide

## For Consumer Crates

Sangha is designed to be consumed by multiple projects in the AGNOS ecosystem. This guide covers integration patterns.

## Adding sangha as a dependency

```toml
[dependencies]
sangha = "1"
```

For minimal builds (no std):
```toml
[dependencies]
sangha = { version = "1", default-features = false }
```

## kiran/joshua — NPC Social Behavior

### Faction dynamics with coalition module

```rust
use sangha::coalition::{CoalitionGame, shapley_value, merge_coalitions, split_coalition};

// Model faction power dynamics
// Each NPC group's coalition value based on combined strength
let game = CoalitionGame::new(n_factions, faction_values)?;
let importance = shapley_value(&game)?;

// Use Shapley values to determine faction leader influence
// Higher Shapley = more critical to winning coalitions
```

### Social networks for NPC relationships

```rust
use sangha::network::{SocialNetwork, barabasi_albert_with_seed};

// Generate NPC social graph with scale-free properties
// (few highly-connected NPCs, many with few connections)
let social_graph = barabasi_albert_with_seed(npc_count, 3, world_seed)?;
```

## agnosai — Multi-Agent Coordination

### Public goods for resource sharing

```rust
use sangha::coordination::{PublicGoodsGame, public_goods_round, free_rider_equilibrium};

// Model shared resource contribution
let game = PublicGoodsGame::new(agent_count, multiplier, endowment)?;
let nash = free_rider_equilibrium(&game)?;
let actual = public_goods_round(&game, &agent_contributions)?;
```

### Collective decision-making

```rust
use sangha::collective::{borda_count, RankedBallot};

// Agents vote on group action
let ballots: Vec<RankedBallot> = agents.iter()
    .map(|a| RankedBallot::new(a.preference_ranking()))
    .collect();
let result = borda_count(&ballots, n_options)?;
```

## bhava — Social Context for Emotion

### Emotional contagion

```rust
use sangha::contagion::{EmotionalState, HatfieldConfig, hatfield_contagion_step};

// Each agent has emotional valence + susceptibility
let states: Vec<EmotionalState> = agents.iter()
    .map(|a| EmotionalState::new(a.mood, a.susceptibility).unwrap())
    .collect();

// Run contagion on the social network's adjacency list
let config = HatfieldConfig::new(mimicry_rate, feedback_strength)?;
let new_states = hatfield_contagion_step(&states, &adjacency, &config, dt)?;
```

## bodh — Group Decision-Making

### Jury theorem for group accuracy

```rust
use sangha::collective::jury_theorem;

// Predict group accuracy from individual competence
let group_accuracy = jury_theorem(individual_accuracy, group_size)?;
```

## Wiring Modules Together

Sangha modules are independent by design. Wire them at the application layer:

```rust
// Network topology feeds into contagion
let network = sangha::network::watts_strogatz(100, 4, 0.1)?;
let new_moods = sangha::contagion::mood_propagation(
    &moods, &network.edges, decay, dt
)?;

// Trust network feeds into reputation
let rep = sangha::trust::reputation_aggregate(&trust_net, target)?;

// Coalition values inform game decisions
let sv = sangha::coalition::shapley_value(&faction_game)?;
```
