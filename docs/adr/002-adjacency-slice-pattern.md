# ADR-002: Adjacency Slice Pattern for Network Functions

**Status**: Accepted
**Date**: 2026-03-30

## Context

Several modules (`contagion`, `trust`) need network topology as input. Importing `network::SocialNetwork` would break the flat architecture (ADR-001). We needed a way to pass graph data without cross-module coupling.

## Decision

Functions that operate on network topology accept `&[Vec<(usize, f64)>]` — a slice of adjacency lists where each entry is `(neighbor_index, weight)`. This matches the internal representation of `SocialNetwork::edges` and `TrustNetwork::relations`.

## Consequences

**Positive**:
- No cross-module imports — modules stay independent
- Consumers can pass any adjacency data, not just `SocialNetwork`
- Easy to convert: `&network.edges` works directly

**Negative**:
- Less type safety — a bare slice doesn't carry `node_count` or invariants
- Functions must validate bounds themselves (neighbor indices in range)

**Mitigation**: All functions that accept adjacency slices validate bounds at entry and return `SanghaError::InvalidNetwork` on out-of-bounds indices.
