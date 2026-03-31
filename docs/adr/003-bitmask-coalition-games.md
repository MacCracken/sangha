# ADR-003: Bitmask Indexing for Coalition Games

**Status**: Accepted
**Date**: 2026-03-30

## Context

Coalition game theory requires evaluating the characteristic function `v(S)` for every subset `S` of players. For `n` players there are `2^n` subsets. We needed an efficient representation.

## Decision

`CoalitionGame::values` is a `Vec<f64>` of length `2^n`, indexed by bitmask. Bit `i` in the mask indicates player `i` is in the coalition. Player count is capped at 20 (2^20 = ~1M entries).

## Consequences

**Positive**:
- O(1) coalition value lookup
- Subset enumeration via bit manipulation (Gosper's hack variant)
- Shapley value computes in O(n * 2^n) — tractable for n <= 20
- Compact, cache-friendly representation

**Negative**:
- Exponential memory: n=20 uses 8MB (1M * 8 bytes)
- Hard limit at 20 players — larger games need approximate methods
- Not human-readable in serialized form

**Trade-off accepted**: 20 players covers the vast majority of faction/coalition scenarios in NPC games and multi-agent systems. Approximate Shapley (Monte Carlo sampling) can be added in a future version for larger games.
