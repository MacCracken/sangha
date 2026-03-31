# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0] - 2026-03-30

### Added

- **`collective` module** — Group decision-making: plurality vote, Borda count, Condorcet winner, majority rule, wisdom of crowds (mean/median/trimmed mean), Condorcet jury theorem
- **`coalition` module** — Coalition game theory: Shapley value, core stability checking, coalition value lookup, faction merge/split operations
- **`coordination` extensions** — Tragedy of the commons (round, Nash equilibrium, social optimum), repeated game discounting, folk theorem threshold for cooperation sustainability
- **`contagion` module** — Emotional/behavioral contagion: Hatfield emotional contagion model (with feedback), SIS epidemiological model, SIS endemic equilibrium, mood propagation (linear diffusion + decay), epidemic threshold via power iteration, emotional convergence check
- **`trust` module** — Trust and reputation: directed trust networks, trust propagation (discounted path product), reputation aggregation (weighted mean), exponential trust decay, betrayal impact
- **`network` extensions** — Barabasi-Albert preferential attachment generator, BFS shortest path, average path length, network density, betweenness centrality (Freeman)
- Ballot permutation validation in `borda_count` and `condorcet_winner`
- `validate()` methods on `CoalitionGame`, `PublicGoodsGame`, `EmotionalState`, `HatfieldConfig` for post-deserialization safety
- `#[non_exhaustive]` on all public structs (`SocialNetwork`, `PayoffMatrix`, `NashEquilibrium`, `SirState`, `Opinion`)
- `watts_strogatz_with_seed()` for reproducible network generation with custom seeds
- Constructors (`new()`) for `PayoffMatrix`, `NashEquilibrium`, `SirState`

### Fixed

- `watts_strogatz` rewiring now propagates `add_edge` errors instead of silently dropping them
- `watts_strogatz` rewiring checks for duplicate edges before adding (no multi-edges)
- `edge_count()` correctly handles self-loops
- `logistic_growth` doc corrected: returns rate of change, not new population
- `has_consensus` now validates `epsilon` (returns `Result`)
- `gini_coefficient` and `lorenz_curve` validate non-negative finite incomes
- `sis_step` renormalizes S+I=1 invariant after Euler integration
- `SisState::new` validates non-negative, finite, s+i=1
- `contagion_threshold` errors on out-of-bounds neighbor indices (consistent with sibling functions)
- `HatfieldConfig::feedback_strength` now implemented in `hatfield_contagion_step`
- Contribution tolerance in `public_goods_round` uses scaled epsilon instead of `f64::EPSILON`

### Changed

- `has_consensus` signature changed from `-> bool` to `-> Result<bool>`
- `SisState::new` signature changed from `-> Self` to `-> Result<Self>`
- Added `#[inline]` to hot-path functions across all modules

## [0.1.0] - 2026-03-29

### Added

- Initial scaffold with all domain modules
- Full test suite with known-good reference values
- Criterion benchmarks
- CI/CD workflows
