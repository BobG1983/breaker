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
| **FrameMutation** | A scripted mutation applied at a specific fixed-update frame during a scenario run. Used in `frame_mutations` to trigger invariant violations at deterministic points in self-test scenarios | `FrameMutation`, `frame_mutations` field in `ScenarioDefinition` |
| **MutationKind** | The enum of mutation operations a `FrameMutation` can apply: `SetBreakerState`, `SetTimerRemaining`, `SpawnExtraEntities`, `MoveBolt`, `TogglePause` | `MutationKind`, `apply_debug_frame_mutations` |
| **TransitionOut** | Game state representing an animated transition out of a completed node (clear animation). Replaces the old 1-frame `NodeTransition` state. | `GameState::TransitionOut`, `spawn_transition_out`, `TransitionDirection::Out` |
| **TransitionIn** | Game state representing an animated transition into the next node (load animation). | `GameState::TransitionIn`, `spawn_transition_in`, `TransitionDirection::In` |
| **TransitionStyle** | Visual style of a node transition animation — either `Flash` (full-screen overlay fades in/out) or `Sweep` (full-screen rect sweeps across screen). Picked randomly from the seeded `GameRng`. | `TransitionStyle::Flash`, `TransitionStyle::Sweep` |
| **ChipOffers** | Transient resource holding the `ChipDefinition`s offered on the chip selection screen for the current visit. Inserted by `generate_chip_offerings` on `OnEnter(ChipSelect)` and consumed by `spawn_chip_select` and `handle_chip_input`. | `ChipOffers`, `generate_chip_offerings` |
| **ChipOffering** | Enum representing a single item on the chip selection screen. Either `Normal(ChipDefinition)` (a standard chip) or `Evolution { ingredients, result }` (an evolution recipe). On boss nodes with eligible recipes, evolution offerings are injected before normal offerings fill remaining slots. | `ChipOffering::Normal`, `ChipOffering::Evolution` |
| **EvolutionRecipe** | A RON-loaded recipe combining chip ingredients into a new chip. Has `ingredients: Vec<EvolutionIngredient>` (each specifying a chip name and minimum stacks required) and `result_definition: ChipDefinition`. | `EvolutionRecipe`, `EvolutionIngredient`, `chips/definition.rs` |
| **EvolutionRegistry** | Resource holding all loaded `EvolutionRecipe`s. Provides `eligible_evolutions(&ChipInventory)` to return recipes whose ingredient requirements are met by the player's current build. Queried by `generate_chip_offerings` on boss nodes. | `EvolutionRegistry`, `chips/resources.rs` |
| **RunStats** | Resource accumulating gameplay statistics for the current run. Tracks nodes cleared, cells destroyed, bumps, perfect bumps, bolts lost, chips collected, evolutions performed, time elapsed, seed, and highlight moments. Displayed on the run-end screen. Also provides `flux_earned()` for Flux calculation. | `RunStats`, `run/resources.rs` |
| **HighlightTracker** | Tracking state resource used for highlight detection. Has two classes of fields: per-node fields (reset by `reset_highlight_tracker` on each `OnEnter(Playing)`) covering consecutive perfect bumps, node bolts lost, cell timestamps, combo/pinball counters; and cross-node fields (persist across node resets, reset only at run start) covering `best_perfect_streak`, `consecutive_no_damage_nodes`, `fastest_node_clear_secs`, `first_evolution_recorded`. | `HighlightTracker`, `reset_highlight_tracker` |
| **HighlightKind** | Enum of 15 memorable run moment categories: `ClutchClear`, `MassDestruction`, `PerfectStreak`, `FastClear`, `FirstEvolution`, `NoDamageNode`, `MostPowerfulEvolution`, `CloseSave`, `SpeedDemon`, `Untouchable`, `ComboKing`, `PinballWizard`, `Comeback`, `PerfectNode`, `NailBiter`. All thresholds are configurable via `HighlightDefaults` / `defaults.highlights.ron`. | `HighlightKind`, `RunHighlight` |
| **RunHighlight** | A single recorded highlight moment. Has `kind: HighlightKind`, `node_index: u32`, and `value: f32` (context-dependent — seconds remaining for `ClutchClear`, streak count for `PerfectStreak`, distance for `CloseSave`, etc.). The number displayed on the run-end screen is capped at `HighlightConfig::highlight_cap` (default 5, RON-configurable). | `RunHighlight`, `RunStats::highlights` |
| **HighlightDefaults** | RON asset type (`defaults.highlights.ron`) holding all highlight detection thresholds and the `highlight_cap`. The `GameConfig` derive macro generates a `HighlightConfig` resource with `From<HighlightDefaults>`. `RunPlugin` initializes `HighlightConfig` via `init_resource` (uses the `Default` impl, which matches the RON file values). The RON file is not wired into `DefaultsCollection` for hot-reload. | `HighlightDefaults`, `HighlightConfig`, `run/definition.rs` |
| **HighlightTriggered** | Message emitted by highlight detection systems each time a memorable moment fires — always emitted for juice/VFX feedback even if the highlight cap is already full. Consumed by `spawn_highlight_text` (run domain) to spawn floating text popups. | `HighlightTriggered`, `run/messages.rs` |

| **Position2D** | Canonical 2D world-space position component from `rantzsoft_spatial2d`. Game systems read and write `Position2D`; `Transform` is derived from it and must never be written directly by game systems. | `Position2D`, `GlobalPosition2D`, `rantzsoft_spatial2d` |
| **Velocity2D** | 2D velocity component from `rantzsoft_spatial2d`. Used alongside the domain-specific bolt velocity. Entities with `ApplyVelocity` marker have `Position2D` advanced by `Velocity2D` each fixed tick. | `Velocity2D`, `ApplyVelocity`, `PreviousVelocity` |
| **Spatial2D** | Marker component from `rantzsoft_spatial2d` that auto-inserts the full set of required spatial components: `Position2D`, `Rotation2D`, `Scale2D`, all `Previous*`, all `Global*`, propagation enums, and `Transform`. | `Spatial2D`, `#[require(Spatial2D)]` |
| **DrawLayer** | Trait from `rantzsoft_spatial2d` that maps a game-defined enum to a Z value for sprite sorting. The game provides `GameDrawLayer` enum implementing `DrawLayer`. `derive_transform` uses the entity's `DrawLayer` component to set `Transform.translation.z`. | `DrawLayer`, `GameDrawLayer`, `derive_transform` |
| **GlobalPosition2D** | Resolved world-space position computed by `compute_globals` from the parent/child hierarchy. Written by the spatial plugin, read by physics and collision systems. | `GlobalPosition2D`, `GlobalRotation2D`, `GlobalScale2D` |
| **CollisionLayers** | Bitmask pair (`membership`, `mask`) from `rantzsoft_physics2d` controlling which entities interact in spatial queries. Godot-style filtering: `self.mask & other.membership != 0` means interaction is possible. | `CollisionLayers`, `rantzsoft_physics2d` |
| **Aabb2D** | Axis-aligned bounding box from `rantzsoft_physics2d`. Defined by center and half-extents. Used for collision detection and quadtree indexing. `#[require(Spatial2D)]` — spawning `Aabb2D` auto-inserts all spatial components. | `Aabb2D`, `CollisionQuadtree` |
| **DistanceConstraint** | Component from `rantzsoft_physics2d` defining a tethered pair of entities with a maximum separation distance. Used by chain bolts to stay tethered to their anchor. `enforce_distance_constraints` in the bolt domain solves violations each tick. | `DistanceConstraint`, `enforce_distance_constraints` |
| **ChainBolt** | A bolt entity spawned tethered to an anchor bolt via `DistanceConstraint`. Spawned by the `ChainHit` amp effect via `SpawnChainBolt` message → `spawn_chain_bolt` system. Despawned when its anchor bolt is lost via `break_chain_on_bolt_lost`. | `SpawnChainBolt`, `spawn_chain_bolt`, `break_chain_on_bolt_lost` |
| **SpawnChainBolt** | Message sent by `handle_chain_bolt` (behaviors/effects) to request spawning a tethered chain bolt. Fields: `anchor: Entity`, `tether_distance: f32`. Consumed by `spawn_chain_bolt` in the bolt domain. | `SpawnChainBolt`, `bolt/messages.rs` |

**Do NOT use generic terms** like "paddle", "ball", "brick", "level", "powerup", or "upgrade" for type names, identifiers, or modules.
