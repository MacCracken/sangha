# Architecture Overview

## Module Map

```
sangha/
├── network.rs     — Social network graphs, Watts-Strogatz, clustering
├── game_theory.rs — Nash equilibria, prisoner's dilemma, strategies
├── opinion.rs     — Bounded confidence, echo chambers, consensus
├── group.rs       — Tuckman stages, social loafing, groupthink
├── population.rs  — Logistic growth, SIR model, herd immunity
├── influence.rs   — Conformity, social proof, Bass diffusion
├── inequality.rs  — Gini coefficient, Lorenz curve
└── error.rs       — SanghaError enum
```

## Consumers

- kiran/joshua: NPC social behavior
- agnosai: multi-agent coordination
- bhava: social context for emotion
