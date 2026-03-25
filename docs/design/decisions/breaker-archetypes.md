# Breaker Types

**Decision**: Aegis/Chrono/Prism are proof-of-concept designs. Shipped breakers may differ.

## Current Breakers

| Breaker | Bolt-Lost Behavior | Identity |
|---------|-------------------|----------|
| Aegis | Lose a life | Lives-based, classic feel |
| Chrono | Time penalty | Time-pressure amplifier |
| Prism | Spawn extra bolt | Multi-bolt chaos |

## Depth

Broader differentiation beyond bolt-lost: different base stats (speed, width, bump strength), different dash/bump properties, and different interaction profiles.

**Upgrade affinities** (some breakers prefer certain chip types) are noted for Phase 7+ but not implemented in the vertical slice.

## Rationale

The goal is building the breaker type *system*, not committing to final designs. The system must support:
- Per-breaker stat overrides (RON-defined)
- Polymorphic bolt-lost dispatch (behavior system)
- Future: per-breaker abilities, passive effects, upgrade affinities

Don't over-invest in specific breaker balance or lore. Focus on system flexibility.
