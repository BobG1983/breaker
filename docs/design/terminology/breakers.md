# Breakers

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Aegis** | Lives-based breaker (3 lives). On bolt lost: `Do(LoseLife)`. On bump (Bumped/EarlyBumped/LateBumped): `Do(SpeedBoost(1.5/1.1/1.1))`. Effects scoped via `On(target: ...)` in `BreakerDefinition`. | `aegis.breaker.ron`, `RootEffect::On`, `Do(LoseLife)` |
| **Chrono** | Time-penalty breaker (no lives). On bolt lost: `Do(TimePenalty(5.0))`. On Bumped: speed boost. | `chrono.breaker.ron`, `Do(TimePenalty)` |
| **Prism** | Multi-bolt breaker (no lives). On bolt lost: `Do(TimePenalty(7.0))`. On PerfectBump: `Do(SpawnBolts())`. | `prism.breaker.ron`, `Do(SpawnBolts {})` |
