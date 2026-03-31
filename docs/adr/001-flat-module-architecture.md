# ADR-001: Flat Module Architecture

**Status**: Accepted
**Date**: 2026-03-29

## Context

Sangha is a computational sociology library covering 12 domain areas (networks, game theory, trust, etc.). We needed to decide between a workspace of sub-crates, nested module hierarchy, or flat file-per-module layout.

## Decision

All domain modules live as flat files at `src/{module}.rs`. No cross-module imports are allowed — every module only imports from `error.rs`.

## Consequences

**Positive**:
- Zero coupling between modules — consumers pick and choose freely
- Adding a new module is a single file + one line in `lib.rs`
- No dependency diamonds or circular import risk
- Easy to understand: one file = one domain

**Negative**:
- Modules can't share internal types (e.g., `contagion` takes `&[Vec<(usize, f64)>]` instead of `&SocialNetwork`)
- Some code patterns repeat across modules (validation, adjacency handling)

**Trade-off accepted**: the duplication cost is low and the coupling prevention is high-value for a library consumed by 4+ different projects.
