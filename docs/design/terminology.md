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
| **Overclock** | Triggered ability — a chip effect defined as a `TriggerChain` | `Overclock`, `ChipEffect::Overclock`, `TriggerChain` |
| **Bump** | Breaker's upward hit | `BumpGrade`, `BumpPerformed` |
| **Aegis** | Lives-based breaker archetype | `aegis.archetype.ron`, `TriggerChain::LoseLife` |
| **Chrono** | Time-penalty breaker archetype | `chrono.archetype.ron`, `TriggerChain::TimePenalty` |
| **Prism** | Multi-bolt breaker archetype | `prism.archetype.ron`, `TriggerChain::SpawnBolt` |
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
| **Tier** | A group of nodes in a run sequence sharing the same difficulty parameters (HP multiplier, timer multiplier, active ratio, introduced cell types) | `TierDefinition`, `tier_index` |
| **DifficultyCurve** | The ordered list of tier definitions loaded from `defaults.difficulty.ron`; drives procedural node sequence generation | `DifficultyCurve`, `DifficultyCurveDefaults` |
| **NodeType** | The category of a single node in the run sequence: Passive (no hostile cells), Active (contains cells that fight back), or Boss (unique encounter with special mechanics) | `NodeType::Passive`, `NodeType::Active`, `NodeType::Boss` |
| **NodePool** | The pool a node layout belongs to — `Passive`, `Active`, or `Boss` — controls which layouts are eligible for each node type | `NodePool`, `NodeLayout.pool`, `NodeLayoutRegistry::get_pool` |
| **NodeSequence** | The full ordered list of node assignments generated from the difficulty curve and run seed; one `NodeAssignment` per node | `NodeSequence`, `NodeAssignment`, `generate_node_sequence` |
| **ChipInventory** | Runtime resource tracking the player's chip build during a run: which chips are held and at what stack level, and which chips have been seen in offerings | `ChipInventory`, `ChipEntry` |
| **TriggerChain** | Recursive enum that encodes the full trigger→effect tree for both archetype behaviors and overclock chips. Trigger wrapper variants (`OnPerfectBump`, `OnImpact`, etc.) nest around leaf effect variants (`Shockwave`, `LoseLife`, `SpawnBolt`, etc.) | `TriggerChain`, `ImpactTarget`, `ChipEffect::Overclock` |
| **ActiveChains** | Runtime resource holding all `TriggerChain`s active for the current run. Populated from the archetype definition on entering Playing, and extended by `handle_overclock` when an overclock chip is selected | `ActiveChains` |
| **ArmedTriggers** | Component attached to a bolt entity when a trigger chain matches a trigger node but the inner chain is not yet a leaf. Carries the remaining chain; evaluated by the next matching bridge system | `ArmedTriggers` |
| **EffectFired** | Observer event fired by bridge systems when a `TriggerChain` fully resolves to a leaf. Carries the leaf `TriggerChain` variant and an optional bolt entity | `EffectFired` |

**Do NOT use generic terms** like "paddle", "ball", "brick", "level", "powerup", or "upgrade" for type names, identifiers, or modules.
