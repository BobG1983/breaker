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
| **Aegis** | Lives-based breaker archetype | `aegis.archetype.ron`, `Consequence::LoseLife` |
| **Chrono** | Time-penalty breaker archetype | `chrono.archetype.ron`, `Consequence::TimePenalty` |
| **Prism** | Multi-bolt breaker archetype | `prism.archetype.ron`, `Consequence::SpawnBolt` |
| **ExtraBolt** | Additional bolt spawned by the Prism archetype on a perfect bump; despawned on loss rather than respawned | `ExtraBolt` |
| **Chip** | Any Amp, Augment, or Overclock (collective term) | `ChipDefinition`, `ChipRegistry`, `ChipSelected` |
| **Rig** | The player's complete build (Breaker + Bolt + Chips + seed + score) | `Rig`, `RigSummary` |
| **Flux** | Meta-progression currency | `Flux`, `FluxReward` |
| **Scenario** | A named automated test run defined in a `.scenario.ron` file — specifies breaker, layout, input strategy, frame limit, and invariants | `ScenarioDefinition`, `ScenarioConfig`, `ScenarioLifecycle` |
| **Invariant** | A runtime assertion checked every frame during a scenario run; any violation fails the run | `InvariantKind`, `ViolationLog`, `ViolationEntry` |
| **Chaos** | Input strategy that injects random game actions each frame using a seeded RNG | `InputStrategy::Chaos`, `ChaosParams` |
| **Scripted** | Input strategy that plays back a deterministic list of frame-action pairs | `InputStrategy::Scripted`, `ScriptedParams`, `ScriptedFrame` |
| **Hybrid** | Input strategy that runs scripted actions for N frames then switches to chaos | `InputStrategy::Hybrid`, `HybridParams` |
| **Recording** | Dev-only system that captures live `GameAction` inputs to a `.scripted.ron` file for later scripted playback | `RecordingConfig`, `--record` flag |

**Do NOT use generic terms** like "paddle", "ball", "brick", "level", "powerup", or "upgrade" for type names, identifiers, or modules.
