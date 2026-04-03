# Cross-Domain Name Collision Research

**Purpose:** Identify `pub` and `pub(crate)` type name collisions across all domains in
`breaker-game/src/` before building a flat `crate::prelude::{components, resources, states,
messages}` re-export module.

**Bevy version:** 0.18 (confirmed from `Cargo.toml`)

**Domains surveyed:** `bolt`, `breaker`, `cells`, `chips`, `effect`, `fx`, `audio`, `input`,
`walls`, `shared`, `state` (which contains `state/run`, `state/menu`, `state/pause`, `state/app`,
`state/transition`, `state/types`), `debug`

---

## Summary

There are **6 hard collisions** — all in builder typestate markers shared between `bolt` and
`breaker`. There are **0 hard collisions** in components, resources, messages, or states. There
are several **near-collisions** worth noting for readability, but none that block flat re-export
for the primary candidate types.

---

## Hard Collisions

All six are `pub` builder typestate markers with identical unqualified names in two domains.
They are **not** components, resources, messages, or states — they live inside `builder::core::types`
modules that are only used within their own builder call chains.

| Name | Path 1 | Path 2 | Category |
|---|---|---|---|
| `Primary` | `breaker-game/src/bolt/builder/core/types.rs` | `breaker-game/src/breaker/builder/core/types.rs` | Typestate marker (builder) |
| `Extra` | `breaker-game/src/bolt/builder/core/types.rs` | `breaker-game/src/breaker/builder/core/types.rs` | Typestate marker (builder) |
| `Unvisual` | `breaker-game/src/bolt/builder/core/types.rs` | `breaker-game/src/breaker/builder/core/types.rs` | Typestate marker (builder) |
| `Rendered` | `breaker-game/src/bolt/builder/core/types.rs` | `breaker-game/src/breaker/builder/core/types.rs` | Typestate marker (builder) |
| `Headless` | `breaker-game/src/bolt/builder/core/types.rs` | `breaker-game/src/breaker/builder/core/types.rs` | Typestate marker (builder) |
| `NoRole` | `breaker-game/src/bolt/builder/core/types.rs` | `breaker-game/src/breaker/builder/core/types.rs` | Typestate marker (builder) |

### Implication for the prelude

These types are internal to the builder typestate call chain. They are `pub` only because the
builder struct itself is `pub` and the typestate must be visible to callers spelling out the full
`BoltBuilder<Primary, ...>` type. They are **not** candidates for a `prelude::components` or
`prelude::resources` flat module.

If a `prelude` module ever needs to re-export builder types (unlikely), it would need aliased names:
- `BoltPrimary` / `BreakerPrimary` instead of bare `Primary`
- `BoltExtra` / `BreakerExtra` instead of bare `Extra`
- etc.

No action needed for the immediate flat prelude goal.

---

## Component Inventory (no collisions)

All `pub` and `pub(crate)` component types across domains. No unqualified name appears in two
domains.

### bolt (`breaker-game/src/bolt/`)
| Name | Visibility | File |
|---|---|---|
| `Bolt` | `pub` | `components/definitions.rs` |
| `PrimaryBolt` | `pub` | `components/definitions.rs` |
| `BoltServing` | `pub` | `components/definitions.rs` |
| `BoltSpawnOffsetY` | `pub` | `components/definitions.rs` |
| `ExtraBolt` | `pub` | `components/definitions.rs` |
| `SpawnedByEvolution` | `pub` | `components/definitions.rs` |
| `PiercingRemaining` | `pub` | `components/definitions.rs` |
| `BoltLifespan` | `pub` | `components/definitions.rs` |
| `BoltBaseDamage` | `pub` | `components/definitions.rs` |
| `BoltDefinitionRef` | `pub` | `components/definitions.rs` |
| `BoltAngleSpread` | `pub` | `components/definitions.rs` |
| `ImpactSide` | `pub` | `components/definitions.rs` — enum, not a component |
| `LastImpact` | `pub` | `components/definitions.rs` |

### breaker (`breaker-game/src/breaker/`)
| Name | Visibility | File |
|---|---|---|
| `Breaker` | `pub` | `components/core.rs` |
| `BreakerBaseY` | `pub` | `components/core.rs` |
| `BreakerReflectionSpread` | `pub` | `components/core.rs` |
| `PrimaryBreaker` | `pub` | `components/core.rs` |
| `ExtraBreaker` | `pub` | `components/core.rs` |
| `BreakerInitialized` | `pub` | `components/core.rs` |
| `DashSpeedMultiplier` | `pub` | `components/dash.rs` |
| `DashDuration` | `pub` | `components/dash.rs` |
| `DashTilt` | `pub` | `components/dash.rs` |
| `BrakeTilt` | `pub` | `components/dash.rs` |
| `BrakeDecel` | `pub` | `components/dash.rs` |
| `SettleDuration` | `pub` | `components/dash.rs` |
| `DashTiltEase` | `pub` | `components/dash.rs` |
| `SettleTiltEase` | `pub` | `components/dash.rs` |
| `BumpState` | `pub` | `components/bump.rs` |
| `BumpFeedbackState` | `pub` | `components/bump.rs` |
| `BumpPerfectWindow` | `pub` | `components/bump.rs` |
| `BumpEarlyWindow` | `pub` | `components/bump.rs` |
| `BumpLateWindow` | `pub` | `components/bump.rs` |
| `BumpPerfectCooldown` | `pub` | `components/bump.rs` |
| `BumpWeakCooldown` | `pub` | `components/bump.rs` |
| `BumpFeedback` | `pub` | `components/bump.rs` |
| `BreakerTilt` | `pub` | `components/movement.rs` |
| `BreakerAcceleration` | `pub` | `components/movement.rs` |
| `BreakerDeceleration` | `pub` | `components/movement.rs` |
| `DecelEasing` | `pub` | `components/movement.rs` |
| `DashState` | `pub` | `components/state.rs` — enum |
| `DashStateTimer` | `pub` | `components/state.rs` |

### cells (`breaker-game/src/cells/`)
| Name | Visibility | File |
|---|---|---|
| `Cell` | `pub` | `components/types.rs` |
| `RequiredToClear` | `pub` | `components/types.rs` |
| `CellTypeAlias` | `pub(crate)` | `components/types.rs` |
| `CellDamageVisuals` | `pub(crate)` | `components/types.rs` |
| `CellWidth` | `pub(crate)` | `components/types.rs` |
| `CellHeight` | `pub(crate)` | `components/types.rs` |
| `CellHealth` | `pub(crate)` | `components/types.rs` |
| `Locked` | `pub(crate)` | `components/types.rs` |
| `LockAdjacents` | `pub(crate)` | `components/types.rs` |
| `CellRegen` | `pub(crate)` | `components/types.rs` |
| `ShieldParent` | `pub(crate)` | `components/types.rs` |
| `OrbitCell` | `pub(crate)` | `components/types.rs` |
| `OrbitAngle` | `pub(crate)` | `components/types.rs` |
| `OrbitConfig` | `pub(crate)` | `components/types.rs` |
| `CellEffectsDispatched` | `pub(crate)` | `components/types.rs` |

### chips (`breaker-game/src/chips/`)
| Name | Visibility | File |
|---|---|---|
| `ChipEntry` | `pub` | `inventory/data.rs` |
| `ChipInventory` | `pub` | `inventory/data.rs` |
| `Recipe` | `pub` | `resources/data.rs` |
| `ChipCatalog` | `pub` | `resources/data.rs` |

### effect (`breaker-game/src/effect/`)

Components on entities (not builder state):

| Name | Visibility | File |
|---|---|---|
| `EffectSourceChip` | `pub` | `core/types/definitions/enums.rs` |
| `BoundEffects` | `pub` | `core/types/definitions/enums.rs` |
| `StagedEffects` | `pub` | `core/types/definitions/enums.rs` |
| `ActiveDamageBoosts` | `pub` | `effects/damage_boost.rs` |
| `ActiveBumpForces` | `pub` | `effects/bump_force.rs` |
| `ActivePiercings` | `pub` | `effects/piercing.rs` |
| `ActiveSpeedBoosts` | `pub` | `effects/speed_boost.rs` |
| `ActiveSizeBoosts` | `pub` | `effects/size_boost.rs` |
| `ActiveVulnerability` | `pub` | `effects/vulnerable.rs` |
| `ActiveQuickStops` | `pub` | `effects/quick_stop.rs` |
| `RampingDamageState` | `pub` | `effects/ramping_damage.rs` |
| `ChainLightningChain` | `pub` | `effects/chain_lightning/effect.rs` |
| `ChainLightningArc` | `pub` | `effects/chain_lightning/effect.rs` |
| `ChainState` | `pub` | `effects/chain_lightning/effect.rs` — enum |
| `GravityWellMarker` | `pub` | `effects/gravity_well/effect.rs` |
| `GravityWellConfig` | `pub` | `effects/gravity_well/effect.rs` |
| `GravityWellSpawnOrder` | `pub` | `effects/gravity_well/effect.rs` |
| `GravityWellSpawnCounter` | `pub` | `effects/gravity_well/effect.rs` |
| `PulseRing` | `pub` | `effects/pulse/effect.rs` |
| `SecondWindWall` | `pub` | `effects/second_wind/system.rs` |
| `ShieldWall` | `pub` | `effects/shield/system.rs` |
| `ShieldWallTimer` | `pub` | `effects/shield/system.rs` |
| `ShieldReflectionCost` | `pub` | `effects/shield/system.rs` |
| `FlashStepActive` | `pub` | `effects/flash_step.rs` |
| `LivesCount` | `pub` | `effects/life_lost.rs` |
| `AnchorActive` | `pub` | `effects/anchor/effect.rs` |
| `AnchorTimer` | `pub` | `effects/anchor/effect.rs` |
| `AnchorPlanted` | `pub` | `effects/anchor/effect.rs` |
| `PhantomBoltMarker` | `pub(crate)` | `effects/spawn_phantom/effect.rs` |
| `PhantomOwner` | `pub(crate)` | `effects/spawn_phantom/effect.rs` |
| `PhantomSpawnOrder` | `pub(crate)` | `effects/spawn_phantom/effect.rs` |
| `PhantomSpawnCounter` | `pub(crate)` | `effects/spawn_phantom/effect.rs` |
| `ShockwaveSource` | `pub(crate)` | `effects/shockwave/effect.rs` |
| `ShockwaveRadius` | `pub(crate)` | `effects/shockwave/effect.rs` |
| `ShockwaveMaxRadius` | `pub(crate)` | `effects/shockwave/effect.rs` |
| `ShockwaveSpeed` | `pub(crate)` | `effects/shockwave/effect.rs` |
| `ShockwaveDamaged` | `pub(crate)` | `effects/shockwave/effect.rs` |
| `ShockwaveDamageMultiplier` | `pub(crate)` | `effects/shockwave/effect.rs` |
| `ShockwaveBaseDamage` | `pub(crate)` | `effects/shockwave/effect.rs` |
| `TetherBoltMarker` | `pub(crate)` | `effects/tether_beam/effect.rs` |
| `TetherChainBeam` | `pub(crate)` | `effects/tether_beam/effect.rs` |
| `TetherChainActive` | `pub(crate)` | `effects/tether_beam/effect.rs` |
| `TetherBeamComponent` | `pub(crate)` | `effects/tether_beam/effect.rs` |
| `AttractionEntry` | `pub(crate)` | `effects/attraction/effect.rs` |
| `ActiveAttractions` | `pub(crate)` | `effects/attraction/effect.rs` |
| `EntropyEngineState` | `pub(crate)` | `effects/entropy_engine/effect.rs` |
| `PiercingBeamRequest` | `pub(crate)` | `effects/piercing_beam/effect.rs` |
| `CircuitBreakerConfig` | `pub(crate)` | `effects/circuit_breaker/effect.rs` |
| `CircuitBreakerCounter` | `pub(crate)` | `effects/circuit_breaker/effect.rs` |
| `ChainBoltMarker` | `pub(crate)` | `effects/chain_bolt/effect.rs` |
| `ChainBoltAnchor` | `pub(crate)` | `effects/chain_bolt/effect.rs` |
| `ChainBoltConstraint` | `pub(crate)` | `effects/chain_bolt/effect.rs` |
| `PulseEmitter` | `pub(crate)` | `effects/pulse/effect.rs` |
| `PulseSource` | `pub(crate)` | `effects/pulse/effect.rs` |
| `PulseRadius` | `pub(crate)` | `effects/pulse/effect.rs` |
| `PulseMaxRadius` | `pub(crate)` | `effects/pulse/effect.rs` |
| `PulseSpeed` | `pub(crate)` | `effects/pulse/effect.rs` |
| `PulseDamaged` | `pub(crate)` | `effects/pulse/effect.rs` |
| `PulseRingDamageMultiplier` | `pub(crate)` | `effects/pulse/effect.rs` |
| `PulseRingBaseDamage` | `pub(crate)` | `effects/pulse/effect.rs` |

### fx (`breaker-game/src/fx/`)
| Name | Visibility | File |
|---|---|---|
| `FadeOut` | `pub(crate)` | `components.rs` |
| `PunchScale` | `pub(crate)` | `components.rs` |

### shared (`breaker-game/src/shared/`)
| Name | Visibility | File |
|---|---|---|
| `BaseWidth` | `pub` | `components.rs` |
| `BaseHeight` | `pub` | `components.rs` |
| `NodeScalingFactor` | `pub` | `components.rs` |
| `CleanupOnNodeExit` | `pub` | `components.rs` |
| `CleanupOnRunEnd` | `pub` | `components.rs` |
| `MinWidth` | `pub` | `size/types.rs` |
| `MaxWidth` | `pub` | `size/types.rs` |
| `MinHeight` | `pub` | `size/types.rs` |
| `MaxHeight` | `pub` | `size/types.rs` |
| `BaseRadius` | `pub` | `size/types.rs` |
| `MinRadius` | `pub` | `size/types.rs` |
| `MaxRadius` | `pub` | `size/types.rs` |
| `ClampRange` | `pub` | `size/types.rs` |

### walls (`breaker-game/src/walls/`)
| Name | Visibility | File |
|---|---|---|
| `Wall` | `pub` | `components.rs` |

### state — ui/hud components
| Name | Visibility | File |
|---|---|---|
| `NodeTimerDisplay` | `pub(crate)` | `state/run/node/hud/components.rs` |
| `SidePanels` | `pub(crate)` | `state/run/node/hud/components.rs` |
| `StatusPanel` | `pub(crate)` | `state/run/node/hud/components.rs` |
| `HighlightPopup` | `pub(crate)` | `state/run/components.rs` |
| `RunEndScreen` | `pub(crate)` | `state/run/run_end/components.rs` |
| `ChipSelectScreen` | `pub(crate)` | `state/run/chip_select/components.rs` |
| `MainMenuScreen` | `pub(crate)` | `state/menu/main/components.rs` |
| `MenuItem` | `pub(crate)` | `state/menu/main/components.rs` — enum |
| `PauseMenuScreen` | `pub(crate)` | `state/pause/components.rs` |
| `PauseMenuItem` | `pub(crate)` | `state/pause/components.rs` — enum |
| `RunSetupScreen` | `pub(crate)` | `state/menu/start_game/components.rs` |

---

## Resource Inventory (no collisions)

### bolt
| Name | Visibility | File |
|---|---|---|
| `BoltRegistry` | `pub` | `registry.rs` |

### breaker
| Name | Visibility | File |
|---|---|---|
| `BreakerRegistry` | `pub` | `registry.rs` |
| `ForceBumpGrade` | `pub` | `resources.rs` |
| `SelectedBreaker` | `pub` | `resources.rs` |

### cells
| Name | Visibility | File |
|---|---|---|
| `CellConfig` | `pub(crate)` | `resources.rs` |
| `CellTypeRegistry` | `pub(crate)` | `resources.rs` |

### chips
| Name | Visibility | File |
|---|---|---|
| `ChipTemplateRegistry` | `pub(crate)` | `resources/data.rs` |
| `EvolutionTemplateRegistry` | `pub(crate)` | `resources/data.rs` |
| `ChipInventory` | `pub` | `inventory/data.rs` |
| `ChipCatalog` | `pub` | `resources/data.rs` |

### input
| Name | Visibility | File |
|---|---|---|
| `DoubleTapState` | `pub` | `resources.rs` |
| `GameAction` | `pub` | `resources.rs` — enum |
| `InputActions` | `pub` | `resources.rs` |
| `InputConfig` | `pub` | `resources.rs` |

### shared
| Name | Visibility | File |
|---|---|---|
| `GameRng` | `pub` | `rng.rs` |
| `PlayfieldConfig` | `pub` | `playfield.rs` |
| `RunSeed` | `pub` | `resources.rs` |

### walls
| Name | Visibility | File |
|---|---|---|
| `WallRegistry` | `pub` | `registry/core.rs` |

### state/run
| Name | Visibility | File |
|---|---|---|
| `RunState` | `pub` | `resources/definitions.rs` |
| `RunStats` | `pub` | `resources/definitions.rs` |
| `RunOutcome` | `pub` | `resources/definitions.rs` — enum |
| `HighlightCategory` | `pub` | `resources/definitions.rs` — enum |
| `HighlightKind` | `pub` | `resources/definitions.rs` — enum |
| `RunHighlight` | `pub` | `resources/definitions.rs` |
| `HighlightTracker` | `pub` | `resources/definitions.rs` |
| `DifficultyCurve` | `pub` | `resources/definitions.rs` |
| `NodeSequence` | `pub` | `resources/definitions.rs` |
| `NodeAssignment` | `pub` | `resources/definitions.rs` |
| `NodeLayoutRegistry` | `pub` | `node/resources/definitions.rs` |
| `ActiveNodeLayout` | `pub` | `node/resources/definitions.rs` |
| `NodeTimer` | `pub` | `node/resources/definitions.rs` |
| `ScenarioLayoutOverride` | `pub` | `node/resources/definitions.rs` |
| `ClearRemainingCount` | `pub` | `node/resources/definitions.rs` |
| `ChipOffers` | `pub` | `chip_select/resources.rs` |
| `ChipOffering` | `pub` | `chip_select/resources.rs` — enum |
| `ChipSelectConfig` | `pub(crate)` | `chip_select/resources.rs` |
| `MainMenuSelection` | `pub(crate)` | `menu/main/resources.rs` |
| `MainMenuConfig` | `pub(crate)` | `menu/main/resources.rs` |
| `RunSetupSelection` | `pub(crate)` | `menu/start_game/resources.rs` |
| `SeedEntry` | `pub(crate)` | `menu/start_game/resources.rs` |
| `PauseMenuSelection` | `pub(crate)` | `pause/resources.rs` |
| `TimerUiConfig` | `pub(crate)` | `run/node/hud/resources.rs` |

### debug
| Name | Visibility | File |
|---|---|---|
| `Overlay` | `pub(crate)` | `resources.rs` — enum |
| `DebugOverlays` | `pub(crate)` | `resources.rs` |
| `LastBumpResult` | `pub(crate)` | `resources.rs` |
| `RecordingConfig` | `pub(crate)` | `recording/resources.rs` |

---

## Message Inventory (no collisions)

### bolt
| Name | Visibility | File |
|---|---|---|
| `BoltSpawned` | `pub` | `messages.rs` |
| `BoltLost` | `pub` | `messages.rs` |
| `BoltImpactBreaker` | `pub(crate)` | `messages.rs` |
| `BoltImpactCell` | `pub(crate)` | `messages.rs` |
| `BoltImpactWall` | `pub(crate)` | `messages.rs` |
| `RequestBoltDestroyed` | `pub(crate)` | `messages.rs` |

### breaker
| Name | Visibility | File |
|---|---|---|
| `BumpGrade` | `pub` | `messages.rs` — enum |
| `BumpPerformed` | `pub` | `messages.rs` |
| `BumpWhiffed` | `pub` | `messages.rs` |
| `BreakerSpawned` | `pub` | `messages.rs` |
| `BreakerImpactCell` | `pub(crate)` | `messages.rs` |
| `BreakerImpactWall` | `pub(crate)` | `messages.rs` |

### cells
| Name | Visibility | File |
|---|---|---|
| `RequestCellDestroyed` | `pub(crate)` | `messages.rs` |
| `CellDestroyedAt` | `pub(crate)` | `messages.rs` |
| `CellImpactWall` | `pub(crate)` | `messages.rs` |
| `DamageCell` | `pub(crate)` | `messages.rs` |

### walls
| Name | Visibility | File |
|---|---|---|
| `WallsSpawned` | `pub(crate)` | `messages.rs` |

### state/run
| Name | Visibility | File |
|---|---|---|
| `RunLost` | `pub` | `messages.rs` |
| `HighlightTriggered` | `pub` | `messages.rs` |
| `ChipSelected` | `pub` | `chip_select/messages.rs` |
| `NodeCleared` | `pub` | `node/messages.rs` |
| `TimerExpired` | `pub` | `node/messages.rs` |
| `ApplyTimePenalty` | `pub` | `node/messages.rs` |
| `ReverseTimePenalty` | `pub` | `node/messages.rs` |
| `CellsSpawned` | `pub` | `node/messages.rs` |
| `SpawnNodeComplete` | `pub` | `node/messages.rs` |

---

## State Inventory (no collisions)

| Name | Visibility | File |
|---|---|---|
| `GameState` | `pub` | `state/types/game_state.rs` |
| `PlayingState` | `pub` | `state/types/playing_state.rs` |

---

## Near-Collisions

These are not hard name collisions but could cause confusion in a flat prelude module.

| Pair | Reason |
|---|---|
| `ShieldParent` (cells) vs `ShieldWall` (effect) | Both "Shield" prefix but different domain and concept |
| `ShieldWall` (effect) vs `SecondWindWall` (effect) | Both "Wall" suffix in the same domain — intra-domain only |
| `CellsPlugin` (cells) vs `ChipsPlugin` (chips) | Similar naming pattern but uniquely named |
| `OrbitConfig` (cells) vs `GravityWellConfig` (effect) | Both "Config" suffix, but fully qualified |
| `WallRegistry` (walls) vs `NodeLayoutRegistry` / `BoltRegistry` / `BreakerRegistry` | All "Registry" suffix — uniquely prefixed |
| `ClearRemainingCount` (state/run) vs `RequiredToClear` (cells) | Related concept, opposite naming direction |
| `ExtraBolt` (bolt component) vs `Extra` (bolt/breaker builder typestate) | Different types, not a prelude concern |
| `PrimaryBolt` / `PrimaryBreaker` (components) vs `Primary` (builder typestates) | Different types, not a prelude concern |

---

## Key Finding

The only types that need aliasing for a flat prelude are the **six builder typestate markers**
(`Primary`, `Extra`, `Unvisual`, `Rendered`, `Headless`, `NoRole`) shared between `bolt` and
`breaker`. Since these are not component/resource/message/state candidates for a flat prelude,
they do **not** block the planned re-export modules. The four target prelude modules
(`prelude::components`, `prelude::resources`, `prelude::states`, `prelude::messages`) can be built
without any aliasing.

---

## Files Surveyed

- `breaker-game/src/bolt/components/definitions.rs`
- `breaker-game/src/bolt/builder/core/types.rs`
- `breaker-game/src/bolt/messages.rs`
- `breaker-game/src/bolt/filters.rs`
- `breaker-game/src/breaker/components/{core,dash,bump,movement,state}.rs`
- `breaker-game/src/breaker/builder/core/types.rs`
- `breaker-game/src/breaker/messages.rs`
- `breaker-game/src/cells/components/types.rs`
- `breaker-game/src/cells/messages.rs`
- `breaker-game/src/cells/resources.rs`
- `breaker-game/src/chips/definition/types.rs`
- `breaker-game/src/chips/inventory/data.rs`
- `breaker-game/src/chips/resources/data.rs`
- `breaker-game/src/effect/core/types/definitions/enums.rs`
- `breaker-game/src/effect/effects/**` (all effect files)
- `breaker-game/src/fx/components.rs`
- `breaker-game/src/input/resources.rs`
- `breaker-game/src/shared/{components,resources,rng,playfield,size/types}.rs`
- `breaker-game/src/walls/components.rs`
- `breaker-game/src/walls/messages.rs`
- `breaker-game/src/state/run/resources/definitions.rs`
- `breaker-game/src/state/run/node/resources/definitions.rs`
- `breaker-game/src/state/run/node/messages.rs`
- `breaker-game/src/state/run/messages.rs`
- `breaker-game/src/state/run/chip_select/resources.rs`
- `breaker-game/src/state/run/chip_select/messages.rs`
- `breaker-game/src/state/menu/**/{resources,components}.rs`
- `breaker-game/src/state/pause/{resources,components}.rs`
- `breaker-game/src/state/types/{game_state,playing_state}.rs`
- `breaker-game/src/debug/resources.rs`
