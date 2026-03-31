# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0] - 2026-03-30

### Added

#### New Modules

- **`collective`** — Group decision-making: `plurality_vote`, `borda_count`, `condorcet_winner`, `majority_rule`, `wisdom_of_crowds` (Mean/Median/TrimmedMean), `jury_theorem` (Condorcet)
- **`coalition`** — Coalition game theory: `shapley_value` (O(n*2^n), n<=20), `is_core_stable`, `coalition_value`, `merge_coalitions`, `split_coalition`; types: `CoalitionGame`, `ShapleyValues`, `StabilityStatus`, `CoalitionStructure`
- **`coordination`** — N-player mechanisms: `PublicGoodsGame`, `public_goods_round`, `free_rider_equilibrium`, `social_optimum`, `sealed_bid_auction` (first/second price), `mechanism_efficiency`, `TragedyOfCommons`, `tragedy_of_commons_round`, `commons_nash_equilibrium`, `commons_social_optimum`, `repeated_game_discount`, `folk_theorem_threshold`
- **`contagion`** — Emotional/behavioral contagion: `hatfield_contagion_step` (mimicry + feedback), `sis_step` (with S+I=1 renormalization), `sis_endemic_equilibrium`, `mood_propagation` (linear diffusion + decay), `contagion_threshold` (power iteration), `emotional_convergence`; types: `EmotionalState`, `SisState`, `HatfieldConfig`
- **`trust`** — Trust and reputation: `TrustNetwork` (directed, asymmetric), `trust_propagation` (bounded BFS, discounted path product), `reputation_aggregate` (weighted mean by |trust|), `trust_decay` (exponential), `betrayal_impact`; types: `TrustRelation`, `TrustNetwork`, `ReputationScore`

#### Network Extensions

- `barabasi_albert` / `barabasi_albert_with_seed` — preferential attachment scale-free generator (Barabasi & Albert 1999)
- `shortest_path` — unweighted BFS, returns full path or None
- `average_path_length` — mean over all reachable pairs, O(n*(n+E))
- `betweenness_centrality` — Freeman (1977), Brandes-style BFS, O(n*(n+E))
- `density` — fraction of possible edges
- `watts_strogatz_with_seed` — reproducible small-world generation

#### Safety & Correctness

- Validated `Deserialize` impls on 7 types: `CoalitionGame`, `PublicGoodsGame`, `TragedyOfCommons`, `EmotionalState`, `SisState`, `HatfieldConfig`, `TrustRelation` — invalid JSON is rejected at deserialization
- `validate()` methods on all types with invariants for non-serde construction paths
- Ballot permutation validation in `borda_count` and `condorcet_winner` — rejects duplicate candidates
- `#[non_exhaustive]` on all public structs and enums
- Constructors (`new()`) for `PayoffMatrix`, `NashEquilibrium`, `SirState`
- `gini_coefficient` and `lorenz_curve` validate non-negative finite incomes

#### Infrastructure

- CI: Windows added to test matrix; feature-gate tests (`--no-default-features`, `--features std`); benchmark job; 3-way version verification (VERSION + Cargo.toml + tag)
- `scripts/coverage-check.sh` — enforces 70% minimum line coverage
- `scripts/bench-history.sh` — generates `benchmarks.md` with 3-point tracking
- `Makefile` — added `coverage-check` target
- `.editorconfig` — Rust 4sp/100col, TOML/YAML 2sp, Makefile tabs
- `codecov.yml` — fixed: ignores tests/, consistent thresholds (5% project, 70% patch)

#### Documentation

- `docs/architecture/overview.md` — full 13-module map, dependency graph, conventions
- `docs/development/roadmap.md` — v1.1/v1.2 planned features
- `docs/adr/001` — flat module architecture
- `docs/adr/002` — adjacency slice pattern (cross-module data without coupling)
- `docs/adr/003` — bitmask coalition games (O(1) lookup, n<=20)
- `docs/guides/usage.md` — patterns for every module
- `docs/guides/integration.md` — per-consumer wiring guide (kiran, agnosai, bhava, bodh)
- Expanded `CONTRIBUTING.md` — Makefile targets, adding-a-module guide, code conventions
- Expanded `SECURITY.md` — design principles, input validation, supply chain

#### Testing & Benchmarks

- 326 tests (was 65): 311 unit + 14 integration + 1 doctest
- 30 benchmarks (was 5) across all 13 modules
- All formulas verified against published literature (12/12 correct or acceptable)

### Fixed

- `watts_strogatz` rewiring propagates `add_edge` errors (was silently dropped)
- `watts_strogatz` rewiring rejects duplicate edges (no multi-edges)
- `edge_count()` correctly handles self-loops
- `logistic_growth` doc corrected: returns rate of change, not new population
- `sis_step` renormalizes S+I=1 after Euler integration (prevents drift)
- `contagion_threshold` errors on out-of-bounds neighbors (was silently ignored)
- `HatfieldConfig::feedback_strength` now implemented (was declared but unused)
- `public_goods_round` contribution tolerance uses scaled epsilon (was `f64::EPSILON`)

### Changed

- `has_consensus` signature: `-> bool` to `-> Result<bool>` (validates epsilon)
- `SisState::new` signature: `-> Self` to `-> Result<Self>` (validates s+i=1)
- `#[inline]` added to hot-path functions across all modules

## [0.1.0] - 2026-03-29

### Added

- Initial scaffold: `network`, `game_theory`, `opinion`, `group`, `population`, `influence`, `inequality`, `error` modules
- Watts-Strogatz small-world generator, clustering coefficient, degree distribution
- Nash equilibria (2x2), prisoner's dilemma, iterated games, tit-for-tat
- Deffuant bounded confidence, echo chamber index, consensus detection
- Tuckman stages, social loafing (Ringelmann), groupthink risk, collective intelligence
- Logistic growth, SIR model, herd immunity threshold
- Conformity (Asch), social proof, Bass diffusion
- Gini coefficient, Lorenz curve
- Criterion benchmarks (5)
- CI/CD workflows (GitHub Actions)
- 65 tests with known-good reference values
