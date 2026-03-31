# Sangha — Claude Code Instructions

## Project Identity

**Sangha** (Sanskrit: community, assembly) — Sociology engine for social networks, game theory, and group dynamics

- **Type**: Flat library crate
- **License**: GPL-3.0
- **MSRV**: 1.89
- **Version**: SemVer 1.0.0

## Consumers

kiran/joshua (NPC social behavior, faction dynamics), agnosai (multi-agent coordination), bhava (social context for emotion), bodh (group decision-making)

## Development Process

### P(-1): Scaffold Hardening (before any new features)

0. Read roadmap, CHANGELOG, and open issues
1. Test + benchmark sweep of existing code
2. Cleanliness check: `cargo fmt --check`, `cargo clippy --all-features --all-targets -- -D warnings`, `cargo audit`, `cargo deny check`, `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps`
3. Get baseline benchmarks (`./scripts/bench-history.sh`)
4. Internal deep review
5. External research
6. Cleanliness check
7. Additional tests/benchmarks
8. Post-review benchmarks
9. Repeat if heavy

### Work Loop / Working Loop (continuous)

1. Work phase
2. Cleanliness check
3. Test + benchmark additions
4. Run benchmarks
5. Internal review
6. Cleanliness check
7. Deeper tests/benchmarks
8. Run benchmarks again
9. If review heavy, return to step 5
10. Documentation
11. Version check
12. Return to step 1

### Key Principles

- Never skip benchmarks
- `#[non_exhaustive]` on ALL public enums
- `#[must_use]` on all pure functions
- `#[inline]` on hot-path functions
- Every type must be Serialize + Deserialize (serde)
- Feature-gate optional modules
- Zero unwrap/panic in library code
- All types must have serde roundtrip tests

## DO NOT

- **Do not commit or push** — the user handles all git operations
- **NEVER use `gh` CLI** — use `curl` to GitHub API only
- Do not add unnecessary dependencies
- Do not break backward compatibility without a major version bump
- Do not skip benchmarks before claiming performance improvements
