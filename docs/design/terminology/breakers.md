# Breakers

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Aegis** | Lives-based breaker (3 lives). On bolt lost: `Fire(LoseLife(...))`. On bump: `Fire(SpeedBoost(...))`. Effects defined as `RootNode::Stamp(target, tree)` entries in `BreakerDefinition`. | `aegis.breaker.ron`, `RootNode::Stamp`, `Fire(LoseLife(...))` |
| **Chrono** | Time-penalty breaker (no lives). On bolt lost: `Fire(TimePenalty(...))`. On bump: speed boost. | `chrono.breaker.ron`, `Fire(TimePenalty(...))` |
| **Prism** | Multi-bolt breaker (no lives). On bolt lost: `Fire(TimePenalty(...))`. On perfect bump: `Fire(SpawnBolts(...))`. | `prism.breaker.ron`, `Fire(SpawnBolts(...))` |
