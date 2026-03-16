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
| **Aegis** | Lives-based breaker archetype | `Aegis`, `AegisPlugin` |
| **Chrono** | Time-penalty breaker archetype | `Chrono`, `ChronoPlugin` |
| **Prism** | Multi-bolt breaker archetype | `Prism`, `PrismPlugin` |
| **ExtraBolt** | Additional bolt spawned by the Prism archetype on a perfect bump; despawned on loss rather than respawned | `ExtraBolt` |
| **Chip** | Any Amp, Augment, or Overclock (collective term) | `ChipDefinition`, `ChipRegistry`, `ChipSelected` |
| **Rig** | The player's complete build (Breaker + Bolt + Chips + seed + score) | `Rig`, `RigSummary` |
| **Flux** | Meta-progression currency | `Flux`, `FluxReward` |

**Do NOT use generic terms** like "paddle", "ball", "brick", "level", "powerup", or "upgrade" for type names, identifiers, or modules.
