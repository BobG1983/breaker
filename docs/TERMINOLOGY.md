# Terminology

The game has its own vocabulary. These terms are used everywhere: code, UI, design docs, player-facing text.

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Breaker** | The paddle | `Breaker`, `BreakerState`, `BreakerPlugin` |
| **Bolt** | The ball | `Bolt`, `BoltSpeed`, `BoltLost` |
| **Cell** | A brick | `Cell`, `CellGrid`, `CellDestroyed` |
| **Node** | A level | `Node`, `NodeTimer`, `NodeLayout` |
| **Amp** | Passive bolt upgrade | `Amp`, `AmpEffect`, `AmpPool` |
| **Augment** | Passive breaker upgrade | `Augment`, `AugmentEffect` |
| **Overclock** | Triggered ability | `Overclock`, `OverclockTrigger` |
| **Bump** | Breaker's upward hit | `BumpGrade`, `BumpPerformed` |
| **Flux** | Meta-progression currency | `Flux`, `FluxReward` |

**Do NOT use generic terms** like "paddle", "ball", "brick", "level", "powerup", or "upgrade" for type names, identifiers, or modules.
