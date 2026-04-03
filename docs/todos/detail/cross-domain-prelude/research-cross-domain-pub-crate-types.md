# Research: Cross-Domain `pub(crate)` Types in `breaker-game/src/`

**Branch**: `refactor/state-folder-structure`  
**Date**: 2026-04-03

---

## Purpose

Identify all `pub(crate)` types that cross domain boundaries — i.e., types defined in one
domain and imported by at least one other domain — as inputs for a cross-domain prelude
(`crate::components`, `crate::resources`, `crate::states`, `crate::messages`).

**Scope**: Production code only. Types used exclusively within their defining domain or only
in `#[cfg(test)]` code are excluded.

**Note on branch state**: This branch (`refactor/state-folder-structure`) has moved the `run`
subdomain from a standalone top-level module (`crate::run`) under `state` (`crate::state::run`).
Several production files still reference `crate::run::...` (e.g., `effect/effects/time_penalty.rs`,
`bolt/systems/spawn_bolt/system.rs`, `breaker/plugin.rs`, `cells/plugin.rs`). These are
in-flight re-path errors on the branch. The actual structural location is `crate::state::run`.

---

## Category 1: Components

Types with `#[derive(Component)]` used across domain boundaries.

### Marker / Entity-Identity Components

| Type | Home Domain | Used By (domains) | Notes |
|------|-------------|-------------------|-------|
| `Bolt` | `bolt::components` | `breaker`, `cells`, `chips`, `effect`, `fx` | Entity-identity marker; used in `With<Bolt>` queries across nearly all domains |
| `Breaker` | `breaker::components::core` | `bolt`, `cells`, `chips`, `effect`, `fx` | Entity-identity marker; used in `With<Breaker>` queries |
| `Cell` | `cells::components::types` | `bolt`, `breaker`, `chips`, `effect`, `state::run` | Entity-identity marker; used in `With<Cell>` queries |
| `Wall` | `wall::components` | `bolt`, `breaker`, `cells`, `chips`, `effect` | Entity-identity marker; used in `With<Wall>` queries |
| `RequiredToClear` | `cells::components::types` | `state::run` | Read by `init_clear_remaining` and `track_node_completion` |

### Bolt Components Used Cross-Domain

| Type | Home Domain | Used By (domains) | Notes |
|------|-------------|-------------------|-------|
| `BoltServing` | `bolt::components::definitions` | `breaker` | `breaker/systems/bump/system.rs` — skip bump if bolt is serving |
| `BoltBaseDamage` | `bolt::components::definitions` | `effect` | Read in `effect/effects/tether_beam` collision system |
| `BoltLifespan` | `bolt::components::definitions` | `effect` | Referenced in spawn_phantom effect reverse |
| `SpawnedByEvolution` | `bolt::components::definitions` | `effect` (via bolt queries) | Damage attribution tracking |
| `PiercingRemaining` | `bolt::components::definitions` | `bolt` (self-referential in collision system) | Also part of `BoltCollisionParams` query used internally |
| `LastImpact` | `bolt::components::definitions` | `bolt` (internal), `effect` | Used by `mirror_protocol` effect which needs last impact side |
| `ExtraBolt` | `bolt::components::definitions` | `bolt` (internal) | Checked in tick_bolt_lifespan |

### Breaker Components Used Cross-Domain

| Type | Home Domain | Used By (domains) | Notes |
|------|-------------|-------------------|-------|
| `BumpFeedbackState` | `breaker::components::bump` | `breaker` (internal filter) | Used in `BumpTriggerFilter` |
| `DashState` | `breaker::components::state` | `breaker` (internal); queried by effect via AnchorActive | Not directly imported by another domain |

### Shared Components (already in `crate::shared`)

| Type | Home Domain | Used By (domains) | Notes |
|------|-------------|-------------------|-------|
| `CleanupOnNodeExit` | `shared::components` | `bolt`, `breaker`, `cells`, `wall`, `effect`, `state::run` | Ubiquitous; already `pub` in `shared` |
| `CleanupOnRunEnd` | `shared::components` | `breaker`, `state::run` | Already `pub` in `shared` |
| `NodeScalingFactor` | `shared::components` | `bolt`, `breaker` | Used in scale sync systems |
| `BaseWidth` | `shared::components` | `breaker` (re-exported), `bolt` | Re-exported by `breaker::components::mod` |
| `BaseHeight` | `shared::components` | `breaker` (re-exported) | Re-exported by `breaker::components::mod` |

### Effect Components Used Cross-Domain

These are defined in `effect::effects::<module>` and imported by other domains in production code.

| Type | Home Domain | Used By (domains) | Notes |
|------|-------------|-------------------|-------|
| `ActivePiercings` | `effect::effects::piercing` | `bolt` | `bolt/systems/bolt_breaker_collision` and `bolt/queries.rs` — resets piercing on breaker hit |
| `ActiveDamageBoosts` | `effect::effects::damage_boost` | `bolt`, `breaker` | `bolt/systems/bolt_cell_collision/system.rs` multiplies damage; `bolt/queries.rs` includes it |
| `ActiveSizeBoosts` | `effect::effects::size_boost` | `bolt`, `breaker` | `bolt/queries.rs`, `breaker/systems/sync_breaker_scale.rs`, `breaker/systems/breaker_cell_collision.rs`, `breaker/systems/breaker_wall_collision.rs`, `breaker/systems/move_breaker/system.rs` |
| `ActiveSpeedBoosts` | `effect::effects::speed_boost` | `bolt`, `breaker` | `bolt/systems/launch_bolt.rs`, `bolt/queries.rs`, `breaker/systems/move_breaker/system.rs` |
| `ActiveVulnerability` | `effect::effects::vulnerable` | `bolt` | `bolt/systems/bolt_cell_collision/system.rs` — multiplies damage when cell is vulnerable |
| `BoundEffects` | `effect::core` (via `pub use core::*`) | `bolt`, `breaker`, `cells`, `chips`, `wall`, `state::run` | Pushed to entities during dispatch; queried by all trigger bridge systems |
| `StagedEffects` | `effect::core` (via `pub use core::*`) | `bolt`, `breaker`, `cells`, `chips`, `wall`, `state::run` | Working set consumed by trigger evaluation |
| `EffectSourceChip` | `effect::core` (via `pub use core::*`) | `effect` (internal only in production) | |
| `AnchorActive` | `effect::effects::anchor` | `breaker` | `breaker/systems/bump/system.rs` — uses it for anchor perfect window multiplier |
| `AnchorPlanted` | `effect::effects::anchor` | `breaker` | `breaker/systems/bump/system.rs` — uses it for anchor bump force multiplier |
| `FlashStepActive` | `effect::effects::flash_step` | `breaker` | `breaker/systems/dash/system.rs` — teleport on reverse-direction input |
| `LivesCount` | `effect::effects::life_lost` | `state::run` | Lives tracker on the breaker, checked by bolt_lost handler |
| `ShieldWall` | `effect::effects::shield` | `bolt` | Second-wind effect queries it; bolt also uses it |
| `ShieldWallTimer` | `effect::effects::shield` | `effect` (internal) | Ticked by shield tick system |
| `ShieldReflectionCost` | `effect::effects::shield` | `effect` (internal) | Read by reflection cost system |

### Wall Components Used Cross-Domain

| Type | Home Domain | Used By (domains) | Notes |
|------|-------------|-------------------|-------|
| `Wall` | `wall::components` | `bolt`, `breaker`, `cells`, `chips`, `effect` | Entity-identity marker |

### Fx Components Used Cross-Domain

| Type | Home Domain | Used By (domains) | Notes |
|------|-------------|-------------------|-------|
| `FadeOut` | `fx::components` | `bolt`, `breaker` | `bolt/systems/bolt_lost_feedback.rs` spawns it; `breaker/systems/bump_feedback.rs` (test imports) |
| `PunchScale` | `fx::components` | (none in production — only fx plugin self-reference) | |

---

## Category 2: Resources

Types with `#[derive(Resource)]` used across domain boundaries.

| Type | Home Domain | Used By (domains) | Notes |
|------|-------------|-------------------|-------|
| `BoltRegistry` | `bolt::registry` | `bolt` (self), `state::run` (via spawn) | Dispatch bolt effects reads it |
| `BreakerRegistry` | `breaker::registry` | `bolt` | `spawn_bolt` reads `BreakerRegistry` + `SelectedBreaker` to find bolt def |
| `SelectedBreaker` | `breaker::resources` | `bolt` | `spawn_bolt/system.rs` reads `SelectedBreaker` to look up definition |
| `ForceBumpGrade` | `breaker::resources` | `breaker` (internal), scenario runner | Scenario resource for test override |
| `CellTypeRegistry` | `cells::resources` | `state::run` | `state/run/node/definition/types.rs` imports it for layout validation |
| `ChipCatalog` | `chips::resources` | `state::run::chip_select` | `chip_select/resources.rs` uses `ChipDefinition`; `dispatch_chip_effects` reads it |
| `ChipInventory` | `chips::inventory` | `state::run::chip_select` | Read by offering system |
| `InputActions` | `input::resources` | `bolt`, `breaker`, `state::run` | `launch_bolt`, `move_breaker`, `bump` system, menu systems |
| `InputConfig` | `input::resources` | `input` (internal) | Key bindings resource |
| `GameRng` | `shared::rng` | `bolt`, `breaker`, `state::run` | `spawn_bolt`, `generate_node_sequence` |
| `RunSeed` | `shared::resources` | `state::run` | At run start to reseed `GameRng` |
| `PlayfieldConfig` | `shared::playfield` | `bolt`, `breaker`, `cells`, `wall`, `effect`, `state::run` | Ubiquitous: used for spawn positions, clamping, wall placement |
| `RunStats` | `state::run::resources` | `state::run` (internal) | Populated by tracking systems from cells/breaker messages |
| `HighlightTracker` | `state::run::resources` | `state::run` (internal) | Tracks highlight detection across node |
| `ChipSelectConfig` | `state::run::chip_select::resources` | `state::run::chip_select` (internal) | Chip selection screen settings |
| `ChipOffers` | `state::run::chip_select::resources` | `state::run::chip_select`, `chips` (indirectly via `ChipSelected` message) | Offerings for current chip select screen |

---

## Category 3: States

Bevy `States` / `SubStates` used across domain boundaries.

| Type | Home Domain | Used By (domains) | Notes |
|------|-------------|-------------------|-------|
| `GameState` | `shared::game_state` | `bolt`, `breaker`, `cells`, `chips`, `effect`, `wall`, `fx`, `input`, `state::*` | Top-level state machine; schedule `OnEnter`/`OnExit` used universally |
| `PlayingState` | `shared::playing_state` | `bolt`, `breaker`, `cells`, `effect`, `fx`, `state::run` | Sub-state of `Playing`; `run_if(in_state(PlayingState::Active))` used widely |

---

## Category 4: Messages

Bevy `#[derive(Message)]` types used across domain boundaries.

### Bolt Messages Imported by Other Domains

| Type | Home Domain | Used By (domains) |
|------|-------------|-------------------|
| `BoltLost` | `bolt::messages` | `breaker` (applies penalty), `effect` (`bridge_bolt_lost`), `fx` (plays audio) |
| `BoltSpawned` | `bolt::messages` | `state::run::node` (spawn coordinator) |
| `BoltImpactBreaker` (pub(crate)) | `bolt::messages` | `breaker` (grade_bump), `effect` (bridge_impact, bridge_impacted) |
| `BoltImpactCell` (pub(crate)) | `bolt::messages` | `cells` (handle_cell_hit? — no, cells gets DamageCell), `effect` (bridge_impact, bridge_impacted), `chips` (if any) |
| `BoltImpactWall` (pub(crate)) | `bolt::messages` | `effect` (bridge_impact, bridge_impacted, shield reflection) |
| `RequestBoltDestroyed` (pub(crate)) | `bolt::messages` | `effect` (bridge_death, bridge_died — reads while entity alive) |

### Breaker Messages Imported by Other Domains

| Type | Home Domain | Used By (domains) |
|------|-------------|-------------------|
| `BumpPerformed` | `breaker::messages` | `effect` (all bump trigger bridges), `state::run` (track_bumps) |
| `BumpWhiffed` | `breaker::messages` | `effect` (bridge_bump_whiff), `state::run` (UI feedback) |
| `BumpGrade` | `breaker::messages` | `effect` (perfect/early/late bump filters), `state::run` (track_bumps) |
| `BreakerSpawned` | `breaker::messages` | `state::run::node` (spawn coordinator) |
| `BreakerImpactCell` (pub(crate)) | `breaker::messages` | `effect` (bridge_impact, bridge_impacted) |
| `BreakerImpactWall` (pub(crate)) | `breaker::messages` | `effect` (bridge_impact, bridge_impacted) |

### Cell Messages Imported by Other Domains

| Type | Home Domain | Used By (domains) |
|------|-------------|-------------------|
| `DamageCell` (pub(crate)) | `cells::messages` | `bolt` (bolt_cell_collision sends it), `effect` (shockwave, explode, chain_lightning, piercing_beam, tether_beam all send it) |
| `CellDestroyedAt` (pub(crate)) | `cells::messages` | `effect` (bridge_cell_destroyed), `state::run` (track_cells_destroyed, track_node_completion, detect_combo_king, detect_mass_destruction) |
| `RequestCellDestroyed` (pub(crate)) | `cells::messages` | `effect` (bridge_death, bridge_died — reads while entity alive) |
| `CellImpactWall` (pub(crate)) | `cells::messages` | `effect` (bridge_impact, bridge_impacted) |

### Wall Messages Imported by Other Domains

| Type | Home Domain | Used By (domains) |
|------|-------------|-------------------|
| `WallMessages` (if any) | `wall::messages` | See below |

### State::Run Node Messages Imported by Other Domains

| Type | Home Domain | Used By (domains) |
|------|-------------|-------------------|
| `NodeCleared` | `state::run::node::messages` | `effect` (`bridge_node_end` — triggers NodeEnd) |
| `ApplyTimePenalty` | `state::run::node::messages` | `effect` (`time_penalty::fire` writes it) |
| `ReverseTimePenalty` | `state::run::node::messages` | `effect` (`time_penalty::reverse` writes it) |
| `CellsSpawned` | `state::run::node::messages` | `state::run::node` (spawn coordinator) |
| `SpawnNodeComplete` | `state::run::node::messages` | `breaker-scenario-runner` |

### State::Run Chip Select Messages Imported by Other Domains

| Type | Home Domain | Used By (domains) |
|------|-------------|-------------------|
| `ChipSelected` | `state::run::chip_select::messages` | `chips` (`dispatch_chip_effects` reads it) |

---

## Category 5: Other (Enums, Traits, Constants, Type Aliases)

### Enums

| Type | Home Domain | Used By (domains) | Notes |
|------|-------------|-------------------|-------|
| `ImpactSide` | `bolt::components::definitions` | `bolt` (internal); `effect` (mirror_protocol uses last_impact.side) | |
| `GameAction` | `input::resources` | `bolt`, `breaker` | `InputActions` consumer enum; `launch_bolt`, `move_breaker`, `bump` system |
| `GameDrawLayer` | `shared::draw_layer` | `bolt`, `breaker`, `cells`, `wall`, `state::run` | Used when spawning entities with render ordering |
| `EffectKind` | `effect::core` (via `pub use core::*`) | `bolt`, `breaker`, `cells`, `chips`, `wall` | Embedded in definitions (RON data) |
| `EffectNode` | `effect::core` (via `pub use core::*`) | `bolt`, `breaker`, `cells`, `chips`, `wall` | Tree node type for effect chains |
| `RootEffect` | `effect::core` (via `pub use core::*`) | `bolt::definition`, `breaker::definition`, `cells::definition`, `wall::definition`, `chips::definition` | All definitions embed `Vec<RootEffect>` |
| `Target` | `effect::core` (via `pub use core::*`) | `bolt`, `breaker`, `cells`, `chips`, `wall` | Used in dispatch target resolution |
| `Trigger` | `effect::core` (via `pub use core::*`) | `chips`, `effect` (internal) | Used in definition RON deserialization and bridge systems |
| `ImpactTarget` | `effect::core` (via `pub use core::*`) | `chips` | Used in chip definition deserialization tests |
| `TriggerContext` | `effect::core` (via `pub use core::*`) | `effect` (internal), `chips` (e2e tests) | Carries entity context from trigger events |
| `BumpGrade` | `breaker::messages` | `effect`, `state::run` | Enum — used in filters and tracking |
| `Rarity` | `chips::definition::types` | `state::run::chip_select` | Used in ChipOffering and offering generation |
| `NodePool` | `state::run::node::definition::types` | `state::run` (internal) | Layout pool type |

### Constants

| Type | Home Domain | Used By (domains) | Notes |
|------|-------------|-------------------|-------|
| `DEFAULT_BOLT_BASE_DAMAGE` | `bolt::resources` | `bolt` (self), `effect` (shockwave damage tests) | Mostly a reference constant |
| `BOLT_LAYER` | `shared::collision_layers` | `bolt`, `breaker`, `wall`, `cells` | Collision bitmask |
| `CELL_LAYER` | `shared::collision_layers` | `bolt`, `cells`, `breaker` | Collision bitmask |
| `WALL_LAYER` | `shared::collision_layers` | `bolt`, `wall`, `breaker` | Collision bitmask |
| `BREAKER_LAYER` | `shared::collision_layers` | `bolt`, `breaker` | Collision bitmask |

### Type Aliases / Query Filters

| Type | Home Domain | Used By (domains) | Notes |
|------|-------------|-------------------|-------|
| `ActiveFilter` | `bolt::filters` | `bolt` (internal collision systems) | `(With<Bolt>, Without<BoltServing>)` |
| `ServingFilter` | `bolt::filters` | `bolt` (internal) | `(With<Bolt>, With<BoltServing>)` |
| `CollisionFilterBreaker` (pub(crate)) | `breaker::filters` | `breaker` (internal) | Keeps bolt and breaker queries disjoint |
| `BoltRadius` (type alias) | `bolt::components::definitions` | `bolt` (internal) | Alias for `shared::size::BaseRadius` |
| `ChipDefinition` | `chips::definition` | `state::run::chip_select::resources`, `state::run` (spawn chip select UI) | Full chip data for display |
| `EvolutionIngredient` | `chips::definition::types` | `state::run::chip_select::resources` | Used in ChipOffering::Evolution |
| `ChipOffering` | `state::run::chip_select::resources` | `state::run::chip_select` (internal), (exported `pub`) | Screen offering type |

### Functions Used Cross-Domain

| Function | Home Domain | Used By (domains) | Notes |
|----------|-------------|-------------------|-------|
| `apply_velocity_formula` | `bolt::queries` | `effect::effects::speed_boost` | Recalculates bolt velocity after speed boost change |

---

## Summary: Cross-Domain Prelude Candidates

The following types are the most widely shared and are the strongest candidates for a
`crate::prelude` or grouped re-export modules.

### `crate::components` prelude candidates

Entity markers (used in `With<X>` filters across 3+ domains):
- `Bolt` (bolt)
- `Breaker` (breaker)
- `Cell` (cells)
- `Wall` (wall)

Effect state components (bolted onto entities, read across bolt/breaker/cells):
- `BoundEffects` (effect)
- `StagedEffects` (effect)
- `ActivePiercings` (effect)
- `ActiveDamageBoosts` (effect)
- `ActiveSizeBoosts` (effect)
- `ActiveSpeedBoosts` (effect)
- `ActiveVulnerability` (effect)
- `AnchorActive`, `AnchorPlanted` (effect — used by breaker bump)
- `FlashStepActive` (effect — used by breaker dash)
- `LivesCount` (effect — used by run)

Shared lifecycle components:
- `CleanupOnNodeExit` (shared)
- `CleanupOnRunEnd` (shared)
- `NodeScalingFactor` (shared)
- `BaseWidth`, `BaseHeight` (shared)

Bolt-specific components used outside bolt domain:
- `BoltServing` (bolt — checked by breaker bump)
- `RequiredToClear` (cells — read by state::run)

### `crate::resources` prelude candidates

- `PlayfieldConfig` (shared — 6+ domains)
- `GameRng` (shared)
- `RunSeed` (shared)
- `InputActions` (input — bolt, breaker)
- `SelectedBreaker` (breaker — bolt)
- `BreakerRegistry` (breaker — bolt)
- `BoltRegistry` (bolt — state::run)
- `CellTypeRegistry` (cells — state::run)
- `ChipCatalog` (chips — state::run::chip_select)
- `ChipInventory` (chips — state::run::chip_select)

### `crate::states` prelude candidates

- `GameState` (shared — all domains)
- `PlayingState` (shared — most domains)

### `crate::messages` prelude candidates

Most-shared messages (read by 2+ foreign domains):
- `BoltLost` (bolt → breaker, effect)
- `BoltSpawned` (bolt → state::run)
- `BoltImpactBreaker` (bolt → breaker, effect)
- `BoltImpactCell` (bolt → effect)
- `BoltImpactWall` (bolt → effect)
- `RequestBoltDestroyed` (bolt → effect)
- `BumpPerformed` (breaker → effect, state::run)
- `BumpWhiffed` (breaker → effect, state::run)
- `BreakerSpawned` (breaker → state::run)
- `BreakerImpactCell` (breaker → effect)
- `BreakerImpactWall` (breaker → effect)
- `DamageCell` (cells → bolt, effect)
- `CellDestroyedAt` (cells → effect, state::run)
- `RequestCellDestroyed` (cells → effect)
- `CellImpactWall` (cells → effect)
- `NodeCleared` (state::run → effect)
- `ApplyTimePenalty` (state::run → effect)
- `ReverseTimePenalty` (state::run → effect)
- `ChipSelected` (state::run::chip_select → chips)

---

## Key Files

- `breaker-game/src/bolt/components/definitions.rs` — Bolt marker and state components
- `breaker-game/src/bolt/messages.rs` — All bolt messages (some pub(crate))
- `breaker-game/src/bolt/queries.rs` — BoltCollisionData; imports ActivePiercings, ActiveDamageBoosts, ActiveSpeedBoosts from effect
- `breaker-game/src/bolt/systems/spawn_bolt/system.rs` — Imports BreakerRegistry, SelectedBreaker from breaker; RunState from run
- `breaker-game/src/bolt/systems/bolt_cell_collision/system.rs` — Imports Cell, CellHealth, DamageCell from cells; ActiveDamageBoosts, ActiveVulnerability from effect
- `breaker-game/src/bolt/systems/dispatch_bolt_effects/system.rs` — Imports Bolt, Breaker, Cell, Wall to resolve RootEffect targets
- `breaker-game/src/breaker/systems/bump/system.rs` — Imports BoltServing, BoltImpactBreaker from bolt; AnchorActive, AnchorPlanted from effect; InputActions from input
- `breaker-game/src/breaker/systems/move_breaker/system.rs` — Imports ActiveSizeBoosts, ActiveSpeedBoosts from effect; InputActions from input
- `breaker-game/src/breaker/systems/breaker_cell_collision.rs` — Imports Cell from cells; ActiveSizeBoosts from effect
- `breaker-game/src/cells/systems/dispatch_cell_effects/system.rs` — Imports Bolt, Breaker, Wall for target resolution; EffectCommandsExt, RootEffect, Target from effect
- `breaker-game/src/effect/effects/speed_boost.rs` — Imports apply_velocity_formula from bolt
- `breaker-game/src/effect/effects/shield/system.rs` — Imports BoltImpactWall from bolt; Wall from wall; PlayfieldConfig from shared
- `breaker-game/src/effect/effects/time_penalty.rs` — Imports ApplyTimePenalty, ReverseTimePenalty from run::node::messages
- `breaker-game/src/effect/triggers/impact/system.rs` — Imports BoltImpactBreaker, BoltImpactCell, BoltImpactWall, BreakerImpactCell, BreakerImpactWall, CellImpactWall
- `breaker-game/src/effect/triggers/death.rs` — Imports RequestBoltDestroyed, RequestCellDestroyed
- `breaker-game/src/effect/triggers/node_end.rs` — Imports NodeCleared from run::node::messages
- `breaker-game/src/chips/systems/dispatch_chip_effects/system.rs` — Imports Bolt, Breaker, Cell, Wall; ChipSelected from state::run::chip_select
- `breaker-game/src/state/run/node/systems/init_clear_remaining.rs` — Imports RequiredToClear from cells
- `breaker-game/src/state/run/node/systems/track_node_completion.rs` — Imports CellDestroyedAt from cells
- `breaker-game/src/state/run/node/tracking/systems/track_bumps.rs` — Imports BumpPerformed, BumpGrade from breaker
- `breaker-game/src/state/run/chip_select/resources.rs` — Imports ChipDefinition, EvolutionIngredient from chips
- `breaker-game/src/shared/mod.rs` — Foundation layer; all `pub` types here are already cross-domain by design

---

## Branch-Specific Notes

The `refactor/state-folder-structure` branch renamed `crate::run` to `crate::state::run` structurally,
but the following production files still contain `crate::run::...` import paths that will fail to compile:
- `breaker-game/src/effect/effects/time_penalty.rs` — `use crate::run::node::messages::{...}`
- `breaker-game/src/bolt/systems/spawn_bolt/system.rs` — `use crate::run::RunState`
- `breaker-game/src/breaker/plugin.rs` — `use crate::run::node::sets::NodeSystems`
- `breaker-game/src/cells/plugin.rs` — `use crate::run::node::sets::NodeSystems`
- `breaker-game/src/state/run/node/tracking/systems/track_cells_destroyed.rs` — `use crate::run::resources::RunStats`
- `breaker-game/src/state/run/node/tracking/systems/track_bumps.rs` — `use crate::run::resources::*`
- `breaker-game/src/state/run/node/systems/track_node_completion.rs` — `use crate::run::node::{...}`
- And several more files in `state/run/**`

These are in-transit import paths. The fully-resolved paths use `crate::state::run::...`.
The prelude design should use `crate::state::run::...` as the canonical path.
