# Architecture Overview

## Design Principles

- **Flat library crate**: all modules at `src/{module}.rs`, no nested crate structure
- **Zero cross-module imports**: modules only import from `error.rs`, never from each other
- **Consumer composability**: users combine modules freely without hidden coupling
- **Validation at boundaries**: all public functions validate inputs and return `Result`
- **Zero panics**: no `unwrap()`, `expect()`, or array indexing without bounds checks in library code

## Module Map

```
sangha/
├── error.rs        — SanghaError enum, validation helpers
├── network.rs      — Social graphs, Watts-Strogatz, Barabasi-Albert, BFS, centrality
├── game_theory.rs  — 2-player Nash equilibria, prisoner's dilemma, iterated games
├── coordination.rs — N-player public goods, auctions, tragedy of commons, folk theorem
├── coalition.rs    — Shapley value, core stability, faction merge/split
├── collective.rs   — Voting (plurality, Borda, Condorcet), jury theorem, wisdom of crowds
├── trust.rs        — Directed trust networks, propagation, reputation, decay, betrayal
├── contagion.rs    — Hatfield emotional contagion, SIS dynamics, mood propagation
├── opinion.rs      — Deffuant bounded confidence, echo chambers, consensus
├── group.rs        — Tuckman stages, social loafing, groupthink risk
├── population.rs   — Logistic growth, SIR model, herd immunity
├── influence.rs    — Conformity (Asch), social proof, Bass diffusion
└── inequality.rs   — Gini coefficient, Lorenz curve
```

## Dependency Graph

```
error.rs ← every other module
(no other cross-module dependencies)
```

## Data Flow

Consumers wire modules together at the application level:

```
network::SocialNetwork → contagion::hatfield_contagion_step (via adjacency slices)
coalition::ShapleyValues → game logic (faction importance)
collective::VoteResult → group decision outcomes
trust::TrustNetwork → reputation-based agent behavior
```

The adjacency list pattern `&[Vec<(usize, f64)>]` is used by `contagion` and `trust` to accept network data without importing `network::SocialNetwork`, preserving module independence.

## Consumers

| Consumer | Uses | For |
|----------|------|-----|
| kiran/joshua | network, game_theory, coalition | NPC social behavior, faction dynamics |
| agnosai | coordination, collective | Multi-agent coordination, group decisions |
| bhava | contagion, opinion, influence | Social context for emotion |
| bodh | collective, group, coalition | Group decision-making |

## Conventions

- `#[non_exhaustive]` on all public enums and structs
- `#[must_use]` on all pure functions
- `#[inline]` on short hot-path functions
- `Serialize + Deserialize` on all types, with serde roundtrip tests
- `validate()` methods on types with fallible constructors (for post-deserialization safety)
