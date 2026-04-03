# Cross-Domain Dependencies in `breaker-game`

**Bevy version**: 0.18.1 (from `breaker-game/Cargo.toml`)  
**Analysis date**: 2026-04-01  
**Scope**: All `use crate::` cross-domain imports in `breaker-game/src/`

---

## Summary of Domains

The crate has 14 top-level modules. `shared` is treated as a foundation layer, not a peer domain.

| Domain | Role |
|--------|------|
| `shared` | State enums, cleanup markers, collision layers, playfield config — no systems |
| `input` | Raw keyboard → `InputActions` resource |
| `wall` | Boundary wall entities, CCD collision surfaces |
| `bolt` | Bolt physics, collision, speed, spawning |
| `breaker` | Paddle mechanics, bump timing, state machine |
| `cells` | Cell HP, damage, destruction, orbit/shield cells |
| `effect` | Data-driven trigger→effect pipeline |
| `chips` | Passive/triggered chip effects, inventory, catalog |
| `run` | Run state, node sequencing, stats, highlights |
| `run::node` | Node layout, timer, cell spawning, completion |
| `fx` | Cross-cutting visuals: fade-out, punch-scale, transitions |
| `ui` | HUD, timer display, `ChipSelected` message |
| `screen` | State registration, menus, chip selection, run-end |
| `audio` | Stub — no systems yet |
| `debug` | egui overlays, hot-reload, recording |

---

## 1. Per-Domain Cross-Domain Import Map

### `shared` (foundation — imported by everything)
- Imports from: nothing (only `bevy::prelude::*`)
- Exports used by all domains:
  - `GameState`, `PlayingState` — state machine types
  - `CleanupOnNodeExit`, `CleanupOnRunEnd`, `EntityScale` — lifecycle markers
  - `PlayfieldConfig`, `PlayfieldDefaults` — playfield dimensions/config
  - `GameRng`, `RunSeed` — global RNG resource
  - `GameDrawLayer` — render layer enum (implements `rantzsoft_spatial2d::DrawLayer`)
  - `color_from_rgb` — color utility
  - `BOLT_LAYER`, `BREAKER_LAYER`, `CELL_LAYER`, `WALL_LAYER` — collision bitmasks

### `input`
- Imports from `shared`: nothing directly (plugin uses `shared::*` indirectly via `crate::`)
- Imports from other domains: **none**
- Exports consumed by others:
  - `input::resources::{InputActions, InputConfig, GameAction}` — consumed by bolt (`launch_bolt`), breaker (`move_breaker`, `update_bump`), screen (`handle_chip_input`, `handle_run_setup_input`, `toggle_pause`), debug (`recording`)

### `wall`
- Imports from `shared`: `CleanupOnNodeExit`, `PlayfieldConfig`, `BOLT_LAYER`, `WALL_LAYER`, `GameDrawLayer`, `GameState`
- Imports from other domains: **none**
- Exports consumed by others:
  - `wall::components::Wall` — used as query filter by bolt (`bolt_wall_collision`), cells (`cell_wall_collision`), effect (`dispatch_wall_effects`, `ResolveOnCommand`), breaker dispatch, chips dispatch, cells dispatch
  - `wall::messages::WallsSpawned` — consumed by `run::node::check_spawn_complete`

### `bolt`
- Imports from `shared`: `GameState`, `PlayingState`, `GameRng`, `CleanupOnNodeExit`, `CleanupOnRunEnd`, `EntityScale`, `BOLT_LAYER`, `BREAKER_LAYER`, `CELL_LAYER`, `WALL_LAYER`, `GameDrawLayer`, `PlayfieldConfig`
- Imports from `breaker`:
  - `breaker::BreakerSystems` (system set — scheduling only)
  - `breaker::BreakerConfig`, `BreakerRegistry`, `SelectedBreaker` — spawn_bolt reads these to locate breaker y-position and bolt definition
  - `breaker::components::Breaker` — query filter in dispatch_bolt_effects
  - `breaker::filters::CollisionFilterBreaker` — bolt_lost and bolt_breaker_collision
  - `breaker::queries::CollisionQueryBreaker` — bolt_breaker_collision
- Imports from `cells`:
  - `cells::components::{Cell, CellHealth}` — bolt_cell_collision
  - `cells::messages::DamageCell` — bolt_cell_collision writes this
- Imports from `effect`:
  - `effect::EffectSystems` (scheduling)
  - `effect::BoundEffects`, `EffectNode` — bolt builder
  - `effect::RootEffect` — BoltDefinition struct field
  - `effect::effects::damage_boost::ActiveDamageBoosts` — bolt queries
  - `effect::effects::piercing::{ActivePiercings, PiercingRemaining}` — bolt queries
  - `effect::effects::speed_boost::ActiveSpeedBoosts` — bolt queries
  - `effect::effects::shield::ShieldActive` — bolt_lost
- Imports from `run::node`:
  - `run::node::ActiveNodeLayout` — apply_entity_scale_to_bolt
  - `run::node::sets::NodeSystems` — scheduling (after NodeSystems::Spawn)
- Imports from `wall`:
  - `wall::components::Wall` — bolt_wall_collision query filter
- Imports from `fx`:
  - `fx::FadeOut` — bolt_lost_feedback
- Exports consumed by others:
  - `bolt::messages::*` — see message table below
  - `bolt::components::Bolt`, `BoltServing`, `BoltBaseDamage`, `BoltDefinitionRef`, `BoltLifespan`, `ExtraBolt`, `PiercingRemaining`, `SpawnedByEvolution` — used by effect, cells, chips, run, breaker, debug
  - `bolt::queries::apply_velocity_formula` — used by effect (gravity_well, speed_boost)
  - `bolt::BoltSystems`, `BoltRegistry`, `BoltDefinition`, `DEFAULT_BOLT_BASE_DAMAGE` — used by effect, screen, debug, run
  - `bolt::resources::DEFAULT_BOLT_BASE_DAMAGE` — used by effect (shockwave, chain_lightning, explode, tether_beam, piercing_beam)

### `breaker`
- Imports from `shared`: `GameState`, `PlayingState`, `PlayfieldConfig`, `BOLT_LAYER`, `BREAKER_LAYER`, `CleanupOnNodeExit`, `CleanupOnRunEnd`, `GameDrawLayer`, `EntityScale`
- Imports from `bolt`:
  - `bolt::BoltSystems` (scheduling)
  - `bolt::messages::BoltImpactBreaker` — grade_bump reads this
  - `bolt::components::BoltServing` — bump system (serving bolt check in forward_bump tests)
- Imports from `cells`:
  - `cells::components::{Cell, CellHealth, ...}` — breaker_cell_collision
- Imports from `effect`:
  - `effect::AnchorActive`, `AnchorPlanted` — bump system uses anchor multipliers
  - `effect::EffectSystems`, `BoundEffects`, `StagedEffects`, `RootEffect`, `Target`, `EffectCommandsExt`, `EffectNode` — dispatch_breaker_effects
  - `effect::effects::size_boost::ActiveSizeBoosts` — bolt_breaker_collision
- Imports from `input`:
  - `input::resources::{GameAction, InputActions}` — move_breaker, bump system
- Imports from `run::node`:
  - `run::node::ActiveNodeLayout`, `run::node::sets::NodeSystems` — apply_entity_scale_to_breaker, scheduling
- Imports from `wall`:
  - `wall::components::Wall` — dispatch_breaker_effects query filter
- Imports from `bolt`:
  - `bolt::components::Bolt` — dispatch_breaker_effects
- Imports from `fx`:
  - `fx::FadeOut` — bump_feedback (in tests)
- Exports consumed by others:
  - `breaker::messages::{BumpPerformed, BumpWhiffed, BreakerSpawned, BreakerImpactCell, BreakerImpactWall}` — consumed by effect, run, ui
  - `breaker::messages::BumpGrade` — consumed by effect triggers (bump/perfect_bump/early_bump/late_bump/bumped/etc.), run (track_bumps)
  - `breaker::BreakerSystems` (system set) — consumed by bolt, effect
  - `breaker::BreakerConfig`, `BreakerDefaults`, `BreakerRegistry`, `SelectedBreaker`, `ForceBumpGrade` — consumed by screen, debug, bolt (spawn_bolt)
  - `breaker::components::{Breaker, BreakerState, BreakerVelocity, BreakerHeight, BreakerWidth}` — consumed by effect (anchor), chips dispatch, cells dispatch, screen
  - `breaker::definition::BreakerDefinition` — consumed by screen (run_setup spawn)

### `cells`
- Imports from `shared`: `CleanupOnNodeExit`, `BOLT_LAYER`, `CELL_LAYER`, `GameState`, `PlayingState`, `GameDrawLayer`
- Imports from `effect`:
  - `effect::effects::shield::ShieldActive` — cells queries, handle_cell_hit
- Imports from `bolt`:
  - `bolt::messages::BoltImpactCell` — cells plugin registers this message; handle_cell_hit reads it (via `BoltImpactCell` → `DamageCell` pipeline from bolt_cell_collision)
- Imports from `wall`:
  - `wall::components::Wall` — dispatch_cell_effects query filter
- Imports from `breaker`:
  - `breaker::components::Breaker` — dispatch_cell_effects query filter
- Imports from `bolt`:
  - `bolt::components::Bolt` — dispatch_cell_effects query filter
- Exports consumed by others:
  - `cells::messages::{DamageCell, RequestCellDestroyed, CellDestroyedAt, CellImpactWall}` — consumed by effect (triggers), run (track_cells_destroyed, track_node_completion)
  - `cells::components::{Cell, CellHealth, RequiredToClear, Locked, CellTypeAlias, ...}` — consumed by bolt, breaker, run::node (spawn_cells_from_layout), chips dispatch, effect (shockwave, chain_lightning, piercing_beam)
  - `cells::CellTypeRegistry`, `CellDefaults` — consumed by screen (loading), run::node (spawn/definition)

### `effect`
- Imports from `shared`: `GameState`, `PlayingState`, `CleanupOnNodeExit`, `BOLT_LAYER`, `CELL_LAYER`, `WALL_LAYER`, `GameRng`, `PlayfieldConfig`
- Imports from `bolt`:
  - `bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall}` — impact trigger bridges
  - `bolt::messages::{BoltLost, RequestBoltDestroyed}` — bolt_lost and death trigger bridges
  - `bolt::messages::BoltSpawned` — (in some tests)
  - `bolt::sets::BoltSystems` — ordering
  - `bolt::components::{Bolt, BoltBaseDamage, BoltDefinitionRef, BoltLifespan, ExtraBolt}` — spawn_bolts, gravity_well, chain_lightning, shockwave, speed_boost, piercing_beam, second_wind
  - `bolt::registry::BoltRegistry` — spawn_bolts
  - `bolt::queries::apply_velocity_formula` — gravity_well, speed_boost
  - `bolt::resources::DEFAULT_BOLT_BASE_DAMAGE` — shockwave, chain_lightning, explode, tether_beam, piercing_beam
- Imports from `breaker`:
  - `breaker::messages::{BumpPerformed, BumpGrade}` — bump/perfect_bump/early_bump/late_bump trigger bridges
  - `breaker::messages::{BreakerImpactCell, BreakerImpactWall}` — impact trigger bridges
  - `breaker::sets::BreakerSystems` — ordering
  - `breaker::components::{Breaker, BreakerState, BreakerVelocity}` — anchor effect
- Imports from `cells`:
  - `cells::messages::{DamageCell, CellDestroyedAt, RequestCellDestroyed}` — cell_destroyed, death trigger bridges; shockwave, chain_lightning, chain_bolt, piercing_beam, explode write DamageCell
  - `cells::components::Cell` — chain_lightning, shockwave, piercing_beam, dispatch_cell_effects, chain_bolt, evaluate (walk_staged_node test)
- Imports from `run::node`:
  - `run::node::messages::{ApplyTimePenalty, ReverseTimePenalty}` — time_penalty effect
- Imports from `wall`:
  - `wall::components::Wall` — ResolveOnCommand, dispatch targets
- Exports consumed by others:
  - `effect::core::*` — `BoundEffects`, `StagedEffects`, `RootEffect`, `EffectNode`, `EffectKind`, `Target`, `Trigger`, `ImpactTarget`, `EffectSourceChip` — consumed by bolt, breaker, cells, chips, wall (dispatch systems), screen
  - `effect::EffectSystems`, `EffectCommandsExt` — consumed by bolt, breaker, cells, wall, chips
  - `effect::effects::{AnchorActive, AnchorPlanted, AnchorTimer}` — consumed by bolt (bump system through effect re-export), breaker
  - `effect::effects::speed_boost::ActiveSpeedBoosts` — consumed by bolt queries, launch_bolt, bolt_lost
  - `effect::effects::piercing::{ActivePiercings, PiercingRemaining}` — consumed by bolt queries
  - `effect::effects::damage_boost::ActiveDamageBoosts` — consumed by bolt queries, shockwave, tether_beam
  - `effect::effects::shield::ShieldActive` — consumed by cells (handle_cell_hit, queries), bolt (bolt_lost)
  - `effect::effects::size_boost::ActiveSizeBoosts` — consumed by breaker (bolt_breaker_collision)
  - `effect::effects::second_wind::SecondWindWall` — consumed by effect::second_wind itself + wall domain types
  - `effect::RootEffect` — consumed by bolt definition, breaker definition, cells definition, chips definition

### `chips`
- Imports from `shared`: `GameState`
- Imports from `ui`:
  - `ui::messages::ChipSelected` — dispatch_chip_effects reads this
- Imports from `effect`:
  - `effect::{BoundEffects, StagedEffects, EffectCommandsExt, EffectNode, RootEffect, Target, Trigger, ImpactTarget}` — chip definition types and dispatch
- Imports from `bolt`:
  - `bolt::components::Bolt` — dispatch_chip_effects resolves Bolt target entities
- Imports from `breaker`:
  - `breaker::components::Breaker` — dispatch_chip_effects resolves Breaker target entities
- Imports from `cells`:
  - `cells::components::Cell` — dispatch_chip_effects resolves Cell target entities
- Imports from `wall`:
  - `wall::components::Wall` — dispatch_chip_effects resolves Wall target entities
- Exports consumed by others:
  - `chips::ChipCatalog` — consumed by screen (generate_chip_offerings, spawn_chip_select, chip resources), run (detect_first_evolution)
  - `chips::ChipDefinition` — consumed by screen (chip_select resources)
  - `chips::inventory::ChipInventory` — consumed by screen (generate_chip_offerings, handle_chip_input)
  - `chips::ChipTemplateRegistry`, `EvolutionTemplateRegistry` — consumed by screen (plugin loading)
  - `chips::systems::propagate_chip_catalog` — consumed by debug (hot_reload)

### `run`
- Imports from `shared`: `GameState`, `PlayingState`, `GameRng`, `RunSeed`, `CleanupOnNodeExit`
- Imports from `breaker`:
  - `breaker::messages::{BumpPerformed, BumpGrade}` — track_bumps
- Imports from `bolt`:
  - `bolt::components::{Bolt, BoltServing}` — detect_nail_biter
  - `bolt::messages::BoltLost` — track_bolts_lost (implied)
- Imports from `cells`:
  - `cells::messages::{CellDestroyedAt, DamageCell}` — track_cells_destroyed, track_evolution_damage
  - `cells::components::RequiredToClear` — init_clear_remaining
  - `cells::CellTypeRegistry` — run::node::definition uses it (test-only)
- Imports from `ui`:
  - `ui::messages::ChipSelected` — track_chips_collected, detect_first_evolution
- Imports from `chips`:
  - `chips::ChipCatalog` — detect_first_evolution reads recipes
- Imports from `fx`:
  - `fx::{FadeOut, PunchScale}` — spawn_highlight_text
- Imports from `wall`:
  - `wall::messages::WallsSpawned` — run::node::check_spawn_complete
- Imports from `bolt`:
  - `bolt::messages::BoltSpawned` — check_spawn_complete
- Imports from `breaker`:
  - `breaker::messages::BreakerSpawned` — check_spawn_complete
- Exports consumed by others:
  - `run::RunState`, `RunStats`, `HighlightKind`, `HighlightTracker`, `RunHighlight`, `RunSeed`, `RunOutcome` — consumed by screen (run_end_screen, run_setup)
  - `run::RunPlugin`, `NodeLayoutRegistry`, `NodeLayout` — consumed by screen (plugin, handle_node_cleared)
  - `run::node::NodeTimer` — consumed by ui (spawn_timer_hud, update_timer_display)
  - `run::node::ActiveNodeLayout` — consumed by bolt, breaker (entity scale), screen (generate_chip_offerings), debug (capture_frame)
  - `run::node::messages::{ApplyTimePenalty, ReverseTimePenalty, NodeCleared, SpawnNodeComplete}` — consumed by effect (time_penalty), run systems internally
  - `run::resources::DifficultyCurveDefaults` — consumed by screen (plugin loading)
  - `run::HighlightConfig` — consumed by screen (run_end_screen)

### `fx`
- Imports from `shared`: `GameState`, `PlayingState`, `GameRng`, `GameState`
- Imports from other game domains: **none**
- Exports consumed by others:
  - `fx::FadeOut` — consumed by bolt (`bolt_lost_feedback`), breaker (`bump_feedback`), run (`spawn_highlight_text`)
  - `fx::PunchScale` — consumed by run (`spawn_highlight_text`)

### `ui`
- Imports from `shared`: `GameState`, `PlayingState`, `CleanupOnNodeExit`, `color_from_rgb`
- Imports from `run::node`:
  - `run::node::NodeTimer` — spawn_timer_hud, update_timer_display
- Imports from other domains: nothing else in production code
- Exports consumed by others:
  - `ui::messages::ChipSelected` — consumed by chips (dispatch_chip_effects), run (track_chips_collected, detect_first_evolution), screen (handle_chip_input)
  - `ui::TimerUiDefaults` — consumed by screen (plugin loading)

### `screen`
- Imports from `shared`: `GameState`, `PlayingState`, `CleanupOnNodeExit`, `CleanupOnRunEnd`, `PlayfieldConfig`, `PlayfieldDefaults`, `RunSeed`, `color_from_rgb`
- Imports from `breaker`:
  - `breaker::BreakerDefaults`, `BreakerRegistry`, `SelectedBreaker` — plugin loading, run_setup screen
  - `breaker::definition::BreakerDefinition` — spawn_run_setup UI card rendering
- Imports from `cells`:
  - `cells::CellDefaults` — plugin loading
- Imports from `input`:
  - `input::InputDefaults`, `InputConfig` — plugin loading, chip/menu input handling
- Imports from `run`:
  - `run::resources::{RunState, RunStats, RunOutcome, HighlightKind, RunHighlight}` — run_end screen display
  - `run::HighlightConfig` — run_end screen
  - `run::node::{ActiveNodeLayout, definition::NodePool}` — generate_chip_offerings
  - `run::NodeLayoutRegistry` — plugin loading
- Imports from `chips`:
  - `chips::ChipCatalog`, `ChipDefinition`, `chips::definition::EvolutionIngredient` — chip_select resources and UI
  - `chips::inventory::ChipInventory` — generate_chip_offerings, handle_chip_input
  - `chips::offering::{OfferingConfig, generate_offerings}` — generate_chip_offerings
  - `chips::ChipTemplateRegistry`, `EvolutionTemplateRegistry` — plugin loading
- Imports from `ui`:
  - `ui::TimerUiDefaults` — plugin loading
  - `ui::messages::ChipSelected` — handle_chip_input writes this
- Imports from `bolt`:
  - `bolt::BoltRegistry` — plugin loading
- Imports from `run`:
  - `run::resources::DifficultyCurveDefaults` — plugin loading
- Exports consumed by others:
  - `screen::ScreenPlugin` — consumed by game.rs plugin group

### `audio`
- Imports from other domains: **none** (stub plugin)
- Exports consumed by others: none

### `debug`
- Imports from `shared`: `GameState`, `PlayingState`
- Imports from `bolt`:
  - `bolt::components::Bolt` — bolt_info_ui
  - `bolt::BoltSystems` — telemetry plugin ordering
- Imports from `breaker`:
  - `breaker::BreakerConfig` — hot_reload propagate_breaker_config, propagate_breaker_changes
  - `breaker::components::{Breaker, BreakerState, ...}` — breaker_state_ui
- Imports from `cells`:
  - `cells::components::*` — propagate_cell_type_changes, hot_reload cell sync
  - `cells::CellTypeRegistry`, `cells::CellTypeDefinition` — hot_reload
- Imports from `input`:
  - `input::resources::InputActions` — input_actions_ui, capture_frame
  - `input::resources::GameAction` — recording resources
- Imports from `run::node`:
  - `run::node::ActiveNodeLayout` — capture_frame
- Imports from `chips`:
  - `chips::systems::propagate_chip_catalog` — hot_reload plugin

---

## 2. Message Ownership and Consumers

| Message | Defined in | Written by | Read by |
|---------|-----------|-----------|---------|
| `BoltSpawned` | `bolt` | `bolt::spawn_bolt` | `run::node::check_spawn_complete` |
| `BoltImpactBreaker` | `bolt` | `bolt::bolt_breaker_collision` | `breaker::grade_bump`, `effect::triggers::impact` |
| `BoltImpactCell` | `bolt` | `bolt::bolt_cell_collision` | `cells::handle_cell_hit` (indirectly via `DamageCell`), `effect::triggers::impact` |
| `BoltImpactWall` | `bolt` | `bolt::bolt_wall_collision` | `effect::triggers::impact`, `effect::second_wind` |
| `BoltLost` | `bolt` | `bolt::bolt_lost` | `effect::triggers::bolt_lost`, `run::track_bolts_lost` |
| `RequestBoltDestroyed` | `bolt` | `bolt::bolt_lost` | `effect::triggers::death`, `bolt::cleanup_destroyed_bolts` |
| `BreakerSpawned` | `breaker` | `breaker::spawn_breaker` | `run::node::check_spawn_complete` |
| `BumpPerformed` | `breaker` | `breaker::grade_bump` | `effect::triggers::{bump, perfect_bump, early_bump, late_bump, bumped, ...}`, `run::track_bumps` |
| `BumpWhiffed` | `breaker` | `breaker::grade_bump` | UI (whiff text spawner) |
| `BreakerImpactCell` | `breaker` | `breaker::breaker_cell_collision` | `effect::triggers::impact`, `effect::triggers::impacted` |
| `BreakerImpactWall` | `breaker` | `breaker::breaker_wall_collision` | `effect::triggers::impact`, `effect::triggers::impacted` |
| `DamageCell` | `cells` | `bolt::bolt_cell_collision`, `effect::{shockwave, chain_lightning, chain_bolt, piercing_beam, explode}` | `cells::handle_cell_hit` |
| `RequestCellDestroyed` | `cells` | `cells::handle_cell_hit` | `effect::triggers::death`, `cells::cleanup_cell` |
| `CellDestroyedAt` | `cells` | `cells::cleanup_cell` | `effect::triggers::cell_destroyed`, `run::track_cells_destroyed`, `run::node::track_node_completion` |
| `CellImpactWall` | `cells` | `cells::cell_wall_collision` | `effect::triggers::impact`, `effect::triggers::impacted` |
| `WallsSpawned` | `wall` | `wall::spawn_walls` | `run::node::check_spawn_complete` |
| `ChipSelected` | `ui` | `screen::handle_chip_input` | `chips::dispatch_chip_effects`, `run::track_chips_collected`, `run::detect_first_evolution` |
| `RunLost` | `run` | `run::handle_timer_expired`, `run::bolt_lost` integration (via timer) | `run::handle_run_lost` |
| `HighlightTriggered` | `run` | `run::highlights::*` | `run::spawn_highlight_text` |
| `NodeCleared` | `run::node` | `run::node::track_node_completion` | `run::handle_node_cleared`, `run::detect_nail_biter`, `run::track_node_cleared_stats` |
| `TimerExpired` | `run::node` | `run::node::tick_node_timer` | `run::handle_timer_expired` |
| `ApplyTimePenalty` | `run::node` | `effect::time_penalty` | `run::node::apply_time_penalty` |
| `ReverseTimePenalty` | `run::node` | `effect::time_penalty` | `run::node::reverse_time_penalty` |
| `CellsSpawned` | `run::node` | `run::node::spawn_cells_from_layout` | `run::node::check_spawn_complete` |
| `SpawnNodeComplete` | `run::node` | `run::node::check_spawn_complete` | scenario runner (external) |

---

## 3. Circular Dependencies

### Confirmed Cycles

**bolt ↔ breaker** (tight bidirectional):
- `bolt` reads `breaker::BreakerConfig`, `BreakerRegistry`, `SelectedBreaker`, `components::Breaker`, `filters::CollisionFilterBreaker`, `queries::CollisionQueryBreaker` (in spawn_bolt, bolt_breaker_collision, bolt_lost)
- `breaker` reads `bolt::BoltSystems`, `bolt::messages::BoltImpactBreaker`, `bolt::components::BoltServing` (in grade_bump, bump system)

**bolt ↔ effect** (tight bidirectional):
- `bolt` reads from `effect::effects::{speed_boost, piercing, damage_boost, shield}`, `effect::EffectSystems`, `effect::BoundEffects`, `effect::EffectNode`
- `effect` reads `bolt::messages::*`, `bolt::components::*`, `bolt::registry::BoltRegistry`, `bolt::queries::apply_velocity_formula`, `bolt::resources::DEFAULT_BOLT_BASE_DAMAGE`

**breaker ↔ effect** (bidirectional):
- `breaker` reads `effect::{AnchorActive, AnchorPlanted, EffectSystems}` and dispatches effects
- `effect` reads `breaker::messages::{BumpPerformed, BreakerImpactCell, BreakerImpactWall}` and `breaker::components::Breaker`

**cells ↔ effect** (bidirectional):
- `cells` reads `effect::effects::shield::ShieldActive`
- `effect` reads `cells::messages::{DamageCell, CellDestroyedAt, RequestCellDestroyed}` and `cells::components::Cell`

**run ↔ fx** (unidirectional in production, but run depends on fx):
- `run` reads `fx::{FadeOut, PunchScale}` for highlight popup spawning
- `fx` does not read from `run` (no cycle here — this is one-way)

**chips ↔ ui** (tight bidirectional):
- `chips::dispatch_chip_effects` reads `ui::messages::ChipSelected`
- `ui::messages::ChipSelected` is defined in `ui`, but sent from `screen`
- `run` also reads `ui::messages::ChipSelected`
- Net: `ui` ← `chips`, `ui` ← `run`, `ui` ← `screen` (all one-way consumers of ui's message)

**run ↔ chips** (bidirectional):
- `run` reads `chips::ChipCatalog` (detect_first_evolution)
- `chips` does not read from `run` directly, but `screen` orchestrates both

**screen is a pure consumer**: screen reads from nearly all domains (breaker, cells, input, run, chips, ui, bolt) but nothing reads from screen.

---

## 4. Shared Types — Candidates for `breaker-shared`

These types are imported by 3 or more domains and have no domain-specific logic:

### Currently in `shared` (already isolated — stays as `breaker-shared`)
- `GameState` — 11 domains import this
- `PlayingState` — 9 domains import this
- `CleanupOnNodeExit`, `CleanupOnRunEnd` — 7+ domains import these
- `PlayfieldConfig` — 8+ domains import this
- `GameRng`, `RunSeed` — 4+ domains import these
- `EntityScale` — bolt, breaker (systems), run::node
- `GameDrawLayer` — bolt, breaker, cells, wall, run::node
- `BOLT_LAYER`, `CELL_LAYER`, `WALL_LAYER`, `BREAKER_LAYER` — 6+ domains import these
- `color_from_rgb` — ui, screen

### Cross-domain component markers used as query filters
These are entity identity markers used across 4+ domains as `With<X>` query filters:
- `Bolt` (from `bolt`) — used in breaker, cells, effect, chips, debug, run
- `Breaker` (from `breaker`) — used in bolt, cells, effect, chips, debug
- `Cell` (from `cells`) — used in bolt, breaker, effect, chips, run
- `Wall` (from `wall`) — used in bolt, effect, chips, cells, breaker

These are the primary source of cross-domain coupling. Any sub-crate split requires these to be accessible from all consumers — they cannot stay in their "owning" crate without creating crate dependencies everywhere.

### Effect core types
The `effect::core::*` types (`BoundEffects`, `StagedEffects`, `RootEffect`, `EffectNode`, `EffectKind`, `Target`, `Trigger`) are imported by 7 domains. They currently live in `effect` but are conceptually shared infrastructure.

### Messages used by 3+ domains
- `DamageCell` (cells) — written by bolt + 5 effect systems, read by cells
- `BoltImpactCell` (bolt) — read by cells + effect
- `BumpPerformed` (breaker) — read by effect (8+ triggers) + run
- `CellDestroyedAt` (cells) — read by effect + run (2 systems)
- `ChipSelected` (ui) — read by chips + run (2 systems) + screen writes it

---

## 5. Isolation Profile

### Most isolated (cleanest split boundary)
| Domain | Incoming deps | Outgoing deps | Verdict |
|--------|--------------|---------------|---------|
| `shared` | 0 | ALL | Foundation layer — split first |
| `input` | 0 | 4 (bolt, breaker, screen, debug) | Highly isolated — straightforward split |
| `audio` | 0 | 0 | Stub, trivially splittable |
| `fx` | 1 (shared) | 3 (bolt feedback, breaker feedback, run highlights) | Mostly isolated — only receives from shared |
| `wall` | 1 (shared) | 4 (bolt, cells, effect, run::node) | Relatively clean |
| `ui` | 2 (shared, run::node) | 3 (chips, run, screen) | Clean if run::node exposes NodeTimer |

### Moderately coupled
| Domain | Notes |
|--------|-------|
| `cells` | Reads from effect (ShieldActive), bolt (BoltImpactCell implicit), shared. Clean output via messages. |
| `run` | Reads from bolt, breaker, cells, ui, chips, fx. Many consumers of run types but outgoing deps manageable. |
| `screen` | Reads from everything but writes to nothing — pure consumer. Could become a "presentation" crate. |
| `debug` | Reads from bolt, breaker, cells, input, run, chips. Dev-only feature gate limits blast radius. |

### Heavily coupled (cannot split without breaking cycles)
| Domain | Notes |
|--------|-------|
| `bolt` ↔ `breaker` | Tight cycle: spawn_bolt reads BreakerConfig/Registry; grade_bump reads BoltImpactBreaker. Cannot split without a shared interface crate. |
| `bolt` ↔ `effect` | Tight cycle: effect effects spawn/modify bolts; bolt uses effect component types. |
| `breaker` ↔ `effect` | Bidirectional: anchor effect reads breaker components; effect triggers read breaker messages. |
| `cells` ↔ `effect` | Bidirectional via ShieldActive + DamageCell/CellDestroyedAt. |
| `chips` ↔ `ui` | chips reads ChipSelected from ui; screen writes ChipSelected. |

---

## 6. Key Structural Observations

### The "entity identity" problem
`Bolt`, `Breaker`, `Cell`, `Wall` are zero-size marker components used as `With<X>` query filters across nearly every domain. They define entity identity. Any split that puts these in separate crates forces every other crate to depend on the "identity" crates — creating a hub-and-spoke topology rather than reducing coupling.

**Option**: Move all four markers to `breaker-shared` (alongside `GameState`, collision layers, etc.). Every domain would then depend on `breaker-shared` for entity identity, which is already the case for `GameState` and `PlayfieldConfig`.

### The `effect` domain straddles a boundary
`effect::core::*` types (`RootEffect`, `BoundEffects`, `StagedEffects`, `EffectNode`, etc.) are used by the definition types in `bolt`, `breaker`, `cells`, and `chips`. This means `bolt::BoltDefinition` has a field of type `Vec<RootEffect>` — so `bolt` currently depends on `effect` for its own definition struct. A split would require either:
1. Moving `RootEffect` and the definition tree types into `breaker-shared`, or
2. Keeping bolt/breaker/cells/chips in the same crate as `effect`

### `screen` as a "presentation shell"
`screen` reads from 8+ domains but is never read by any of them. It is an ideal candidate for a separate `breaker-screen` or `breaker-ui` crate that depends on everything else.

### `debug` is already feature-gated
All `debug` production code is under `#[cfg(feature = "dev")]`. It has no production consumers. This makes it a clean candidate for a `breaker-dev` crate or keeping it as a feature-gated module.

### `run::node` is a sub-domain of `run`
`run::node` has its own plugin, messages, and systems. Several domains import `run::node::NodeTimer`, `ActiveNodeLayout`, and `NodeSystems` directly. If `run` is split, `run::node` types must be re-exported cleanly.

---

## 7. Recommended Split Feasibility

### Tier 1 — Splittable now (no cycles to resolve)
1. `breaker-shared` — already exists as `shared`; move entity markers (`Bolt`, `Breaker`, `Cell`, `Wall`) here too
2. `breaker-input` — `input` domain; zero incoming deps
3. `breaker-fx` — `fx` domain; minimal incoming deps, no cycles
4. `breaker-audio` — stub; trivial

### Tier 2 — Splittable after Tier 1 (need `breaker-shared` to expose entity markers)
5. `breaker-wall` — depends on shared only
6. `breaker-ui` — depends on shared + run::node::NodeTimer
7. `breaker-cells` — depends on shared + effect::ShieldActive; effect cycle is manageable if ShieldActive moves to shared

### Tier 3 — Requires interface crate to break cycles
8. `breaker-effect-core` — extract `RootEffect`, `BoundEffects`, `StagedEffects`, `EffectNode`, `EffectKind`, `Target`, `Trigger` into a standalone types crate
9. `breaker-bolt` + `breaker-breaker` — after bolt ↔ breaker cycle is resolved via `BreakerSpawnParams` trait or similar interface
10. `breaker-chips` — after effect-core is separate

### Tier 4 — Aggregator crates (compile-time benefit from Tier 1-3 splits)
11. `breaker-run` — depends on cells messages, bolt components, breaker messages, fx, chips catalog
12. `breaker-screen` — pure consumer; depends on everything; natural leaf crate
13. `breaker-debug` — feature-gated; depends on everything; natural leaf crate

---

## Key Files

- `breaker-game/src/lib.rs` — module declarations for all 14 domains
- `breaker-game/src/shared/mod.rs` — foundation types (the embryo of `breaker-shared`)
- `breaker-game/src/effect/core/` — shared effect infrastructure used across 7 domains
- `breaker-game/src/bolt/builder/core.rs` — bolt builder imports from shared + effect (defines the bolt↔effect coupling)
- `breaker-game/src/bolt/systems/spawn_bolt/system.rs` — crosses bolt↔breaker boundary (reads BreakerRegistry)
- `breaker-game/src/breaker/systems/bump/system.rs` — crosses breaker↔bolt boundary (reads BoltImpactBreaker)
- `breaker-game/src/run/node/systems/check_spawn_complete.rs` — crosses run::node into bolt/breaker/wall/cells (reads all 4 spawn messages)
- `breaker-game/src/chips/systems/dispatch_chip_effects/system.rs` — crosses chips into bolt/breaker/cells/wall/ui
- `breaker-game/src/screen/plugin.rs` — the widest consumer; imports from 8+ domains for registry loading
