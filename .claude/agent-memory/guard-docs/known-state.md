---
name: Known State
description: Intentionally forward-looking docs, known gaps, scenario runner architecture, drift patterns
type: reference
---

## Intentionally Forward-Looking Docs (Do Not Flag as Drift)
- `docs/architecture/state.md` — lists `MetaProgression` state that exists in code but screen is not yet built
- `docs/plan/done/phase-2/phase-2e-chrono-and-prism.md` — checklist has 3 open items (Chrono/Prism bolt-loss visual indicators, all three playable). Known incomplete accepted as Phase 2 closed.

## Known Gaps (Accepted for Now)
- Phase 2e checklist: Bolt-loss visual indicators for Chrono and Prism not yet built — left unchecked

## Scenario Runner Architecture (do not re-flag)
- `breaker-scenario-runner/` is a workspace peer — documented in `plugins.md` workspace layout
- `ScenarioLayoutOverride` resource lives in `breaker-game/src/run/node/resources.rs` (not shared/) — allows scenario runner to bypass run setup
- `debug/recording/` sub-domain exists in `breaker-game/src/debug/` — captures live inputs
- `validate_pass` logic: if `expected_violations: Some(...)` the scenario is a self-test — violations must match exactly
- Log capture filter covers both `breaker` and `breaker_scenario_runner` targets
- CI: `.github/workflows/ci.yml` has `test` (3 platforms) and `scenarios` (Linux headless) jobs

## Spawn Coordination Architecture (do not re-flag)
- `SpawnNodeComplete` is a real active message sent by `check_spawn_complete` in `run/node/` — consumed by scenario runner for baseline entity count sampling
- Spawn signals: `BreakerSpawned` (breaker), `BoltSpawned` (bolt), `CellsSpawned` (run/node), `WallsSpawned` (wall) — all consumed by `check_spawn_complete`
- `check_spawn_complete` uses a `Local<SpawnChecklist>` bitfield — resets after firing to allow multi-node runs
- All 5 of these messages are now documented in `docs/architecture/messages.md`

## NodeSystems Set (do not re-flag)
- `NodeSystems` enum lives in `run/node/sets.rs` with variants: Spawn, TrackCompletion, TickTimer, ApplyTimePenalty, InitTimer
- Used cross-domain: `run/plugin.rs` orders `handle_node_cleared` and `handle_timer_expired` against it
- Now documented in `docs/architecture/ordering.md` and `docs/architecture/plugins.md`

## BreakerSystems::Reset (do not re-flag)
- `BreakerSystems::Reset` tags `reset_breaker` in `breaker/plugin.rs` OnEnter(Playing)
- Intra-domain only — no cross-domain consumers currently
- Added to ordering.md defined sets table with note "intra-domain only"

## Chips Domain Architecture (do not re-flag)
- `chips/` has `definition.rs` (content data types: ChipDefinition, AmpEffect, AugmentEffect, ChipEffect, TriggerChain, ImpactTarget, Rarity, ChipEffectApplied)
- `chips/effects/` promoted directory with per-effect observer handlers (mirrors behaviors/effects/ pattern — note: behaviors/consequences/ was deleted in refactor/unify-behaviors; behaviors/effects/ is the current name)
- `ChipEffectApplied { effect, max_stacks }` is `#[derive(Event)]` (observer trigger) — lives in `chips/definition.rs` (moved from chips/messages.rs in refactor/phase4-wave1-cleanup). Consistent with behaviors domain pattern. No longer flagged.
- `ChipEffectApplied` documented in messages.md Observer Events table

## Phase 4 Wave 4 Architecture (as of 2026-03-23, do not re-flag)

### Chip Evolution (4h)
- `ChipOffering` enum in `screen/chip_select/resources.rs`: `Normal(ChipDefinition)` and `Evolution { ingredients: Vec<EvolutionIngredient>, result: ChipDefinition }`
- `EvolutionRecipe` / `EvolutionIngredient` in `chips/definition.rs` — both `Asset + TypePath + Deserialize`; ingredient fields: `chip_name: String`, `stacks_required: u32`; recipe field: `result_definition: ChipDefinition`
- `EvolutionRegistry` in `chips/resources.rs` — flat `Vec<EvolutionRecipe>`; `eligible_evolutions(&ChipInventory)` for lookup
- Evolution is presented via existing ChipSelect screen (not a separate screen) — boss nodes inject `ChipOffering::Evolution` before normal chips in `generate_chip_offerings`
- `handle_chip_input` consumes ingredient stacks for `ChipOffering::Evolution` on confirm
- No RON data files in `assets/evolutions/` yet — infrastructure in place, data authoring pending
- `EvolutionRegistry` NOT in `DefaultsCollection` / loading screen — only inserted if explicitly added; `generate_chip_offerings` uses `Option<Res<EvolutionRegistry>>`
- No `EvolutionConsumesIngredients` scenario invariant yet — described in plan but not yet implemented in runner

### Run Stats & Highlights (4i) — UPDATED for memorable moments wave (2026-03-23)
- `RunStats` resource in `run/resources.rs` — counters: nodes_cleared, cells_destroyed, bumps_performed, perfect_bumps, bolts_lost, chips_collected (Vec<String>), evolutions_performed, time_elapsed, seed; plus `highlights: Vec<RunHighlight>`
- `HighlightTracker` resource in `run/resources.rs` — per-node AND cross-node transient tracking fields; reset by `reset_highlight_tracker` (per-node fields only)
- `HighlightKind` enum (15 variants): ClutchClear, MassDestruction, PerfectStreak, FastClear, FirstEvolution, NoDamageNode, MostPowerfulEvolution, CloseSave, SpeedDemon, Untouchable, ComboKing, PinballWizard, Comeback, PerfectNode, NailBiter
- `HighlightDefaults` in `run/definition.rs` — `#[derive(Asset, TypePath, Deserialize, GameConfig)]` → generates `HighlightConfig` resource via `#[game_config(name = "HighlightConfig")]`; RON file `assets/config/defaults.highlights.ron` exists (tested via `include_str!`); NOT in `DefaultsCollection` — not hot-reload wired
- `HighlightConfig` is `init_resource`'d in `RunPlugin.build()` — uses `Default` impl (matches `defaults.highlights.ron` values). Fields: clutch_clear_secs, fast_clear_fraction, perfect_streak_count, mass_destruction_count, mass_destruction_window_secs, combo_king_cells, pinball_wizard_bounces, speed_demon_secs, close_save_pixels, comeback_bolts_lost, nail_biter_pixels, untouchable_nodes, highlight_cap
- `HighlightTriggered { kind: HighlightKind }` message in `run/messages.rs` — registered by `RunPlugin`; emitted by all detection systems; consumed by `spawn_highlight_text` for in-game popups
- Stats systems in `run/plugin.rs` FixedUpdate (PlayingState::Active): `track_cells_destroyed`, `track_bumps`, `track_bolts_lost`, `track_time_elapsed`, `track_node_cleared_stats`, `detect_mass_destruction`, `detect_close_save`, `detect_combo_and_pinball`, `detect_nail_biter`
- `detect_close_save` is `.after(BreakerSystems::GradeBump)` (updated in C7-R; previously was .after(BoltSystems::BreakerCollision))
- `detect_nail_biter` is `.after(NodeSystems::TrackCompletion)` (fires on node clear)
- `track_chips_collected` + `detect_first_evolution` run in `Update` during `GameState::ChipSelect`
- `reset_highlight_tracker` + `capture_run_seed` run on `OnEnter(GameState::Playing)` — both unordered
- `track_node_cleared_stats` is `.after(NodeSystems::TrackCompletion)` (already in ordering.md)
- `spawn_highlight_text` — run domain system in `run/systems/spawn_highlight_text.rs`; reads `HighlightTriggered` messages and spawns `Text2d` entities with `FadeOut` + `CleanupOnNodeExit`; imported in `run/plugin.rs` use block but NOT registered in any schedule (wiring gap — system and tests complete, plugin.rs wiring pending)
- Run-end screen reads `Option<Res<HighlightConfig>>` (not required — graceful fallback if absent)
- New invariants: `ChipStacksConsistent` (chip stacks never exceed max_stacks) and `RunStatsMonotonic` (stat counters never decrease) — both in runner `InvariantKind` and `checkers/`

### Release Infrastructure (4j)
- `.github/workflows/release.yml` — builds macOS ARM64, Windows x64, Linux x64; creates GitHub Release; pushes to itch.io (rantzgames/breaker) on tag push; also supports `workflow_dispatch` (build + release only, no itch.io)
- itch.io channels: `mac`, `windows`, `linux`
- `CHANGELOG.md` exists at repo root — release workflow reads first section for release notes

## Phase 4 Wave 3 Architecture (as of 2026-03-22, do not re-flag)

### TransitionOut / TransitionIn states
- `GameState::NodeTransition` DELETED — replaced by `GameState::TransitionOut` + `GameState::TransitionIn`
- Inter-node flow: `Playing → TransitionOut → ChipSelect → TransitionIn → Playing`
- `handle_node_cleared` (run domain) transitions to `TransitionOut` on node clear
- `fx/plugin.rs`: `OnEnter(TransitionOut)` → `spawn_transition_out`; `animate_transition` drives timer and sets `NextState(ChipSelect)` on completion
- `handle_chip_input` (screen/chip_select) transitions to `TransitionIn` on confirm
- `run/plugin.rs`: `OnEnter(TransitionIn)` → `advance_node`
- `fx/plugin.rs`: `OnEnter(TransitionIn)` → `spawn_transition_in`; `animate_transition` sets `NextState(Playing)` on completion
- `TransitionStyle` (Flash/Sweep), `TransitionDirection` (Out/In), `TransitionTimer`, `TransitionOverlay` in `fx/transition.rs`
- `TransitionDefaults`/`TransitionConfig` in `fx/transition.rs` — uses `Default` impl directly (no RON file yet)
- `complete_transition_out.rs` still exists in `run/systems/` but is NOT registered in `run/plugin.rs` — dead code stub

### Chip Offering System (4f)
- `generate_chip_offerings` system in `screen/chip_select/systems/` — runs `OnEnter(ChipSelect)` before `spawn_chip_select`
- `ChipOffers` resource in `screen/chip_select/resources.rs` — transient, inserted per chip-select visit
- `OfferingConfig` + `generate_offerings` in `chips/offering.rs`
- New invariants: `OfferingNoDuplicates`, `MaxedChipNeverOffered` — both in `InvariantKind` and checked in `breaker-scenario-runner/src/invariants/checkers/`
- 13 chip RON files: 5 amps (`amps/`), 4 augments (`augments/`), 4 overclocks (`overclocks/`) — 4c.2 complete

## Phase 4 Wave 1 Status (as of 2026-03-19)
- 4a (Seeded RNG): DONE — moved to `docs/plan/done/phase-4/phase-4a-seeded-rng.md`
- 4b (Chip Effect System): DONE — 4b.1 types/stacking + 4b.2 per-domain consumption both complete. Spec file stays at active location (no separate done file). index.md updated.
- `docs/plan/index.md` 4a link fixed to point to done/ location

## Phase 4b.2 Architecture (do not re-flag)
- `BoltHitCell` now has `{ cell: Entity, bolt: Entity }` — bolt field added for DamageBoost/Piercing lookahead
- `BASE_BOLT_DAMAGE: u32 = 10` constant lives in `shared/mod.rs` — used by cells (handle_cell_hit) and physics (bolt_cell_collision)
- `PiercingRemaining` component lives in `chips/components.rs` — tracks remaining pierces per wall-bounce cycle
- `width_boost_visual` system registered in breaker plugin Update schedule — visual only, no cross-domain ordering needed
- Physics reads `CellHealth` (cells domain) and `DamageBoost`, `Piercing`, `PiercingRemaining` (chips domain) for pierce lookahead
- Cells reads `DamageBoost` (chips domain) from bolt entity for damage calculation
- These cross-domain reads are documented in plugins.md under "Chip Effect — Justified Cross-Domain Component Reads"
- `definition.rs` is now documented as optional canonical layout file in layout.md
- `docs/architecture/content.md` fully rewritten to reflect implemented pattern (was "not yet implemented")

## Phase 4 Wave 2 Architecture (do not re-flag)

### BreakerSystems::GradeBump (do not re-flag)
- `BreakerSystems::GradeBump` is a real set variant in `breaker/sets.rs` — tags `grade_bump` system
- Cross-domain consumers: `behaviors/plugin.rs` orders `bridge_bump` and `bridge_bump_whiff` `.after(BreakerSystems::GradeBump)`
- Added to ordering.md defined sets table and FixedUpdate chain

### bridge_bump_whiff (do not re-flag)
- `bridge_bump_whiff` is a real bridge system in `behaviors/bridges.rs` — reads `BumpWhiffed`, fires `EffectFired`
- Runs `.after(BreakerSystems::GradeBump).in_set(BehaviorSystems::Bridge)`
- NOTE: previously fired `ConsequenceFired` — now fires `EffectFired` after TriggerChain unification

### Phase 4 Wave 2 Completion (as of 2026-03-19)
- 4c.1 (Rarity enum + ChipInventory): DONE — `Rarity` in `chips/definition.rs`, `ChipInventory` in `chips/inventory.rs`
- 4e.1 (Tier data structures + difficulty curve): DONE — `run/difficulty.rs` (TierDefinition, DifficultyCurve, NodeType, DifficultyCurveDefaults, TierNodeCount)
- 4e.2 (Procedural sequence generation): DONE — `run/systems/generate_node_sequence.rs` (NodeAssignment, NodeSequence, generate_node_sequence)
- 4e.3 (Lock + Regen cell types): DONE — `cells/components.rs` (Locked, LockAdjacents, CellRegen); systems `check_lock_release`, `tick_cell_regen`
- 4e.4 (Layout pool support): DONE — `NodePool` in `run/node/definition.rs`, `pools` HashMap in `NodeLayoutRegistry`; `generate_node_sequence_system` registered `OnExit(MainMenu).after(reset_run_state)`
- index.md and phase-4/index.md updated accordingly

### CellTypeDefinition hp field (do not re-flag)
- `CellTypeDefinition.hp` is `f32`, not `u32` — fixed in data.md and content.md

### ChipInventory layout (do not re-flag)
- `chips/inventory.rs` is a standalone resource file (not canonical category — it's a domain-specific resource for tracking the player build)
- Registered in `ChipsPlugin` as `init_resource::<ChipInventory>()`
- Also cleared in `reset_run_state` — chips domain resource touched by run domain at run start (intentional cross-domain resource write in init system)

## New Chip Effects (as of 2026-03-19 session 5) (do not re-flag)
- `AmpEffect::ChainHit(u32)` and `AmpEffect::SizeBoost(f32)` added to `chips/definition.rs`
- `ChainHit` and `BoltSizeBoost` components in `chips/components.rs`
- `handle_chain_hit` and `handle_bolt_size_boost` observers registered in `ChipsPlugin`
- `ChainHit` and `BoltSizeBoost` are stamped by observers but NOT yet consumed by any production gameplay system (physics, cells, bolt) — NOT cross-domain reads yet, not added to plugins.md cross-domain section
- `content.md` already documents these correctly (AmpEffect enum and component list updated)
- SUPERSEDED BY TRIGGERCHAIN UNIFICATION: `behaviors/consequences/` directory deleted; replaced by `behaviors/effects/` with `life_lost`, `time_penalty`, `spawn_bolt`, `shockwave` handlers
- `BoltSpeedBoost` is now a `TriggerChain` leaf variant — no longer a separate file

## SpeedBoost Generalization (merged into develop 2026-03-21) (do not re-flag)
- `TriggerChain::BoltSpeedBoost` renamed to `TriggerChain::SpeedBoost { target: SpeedBoostTarget, multiplier: f32 }`
- `BumpPerformed.multiplier` field deleted — message is now `{ grade: BumpGrade, bolt: Entity }` only
- `apply_bump_velocity` system deleted from bolt domain entirely
- `BumpPerfectMultiplier` and `BumpWeakMultiplier` components deleted from breaker domain
- SpeedBoost is now a normal `TriggerChain` leaf effect handled by `handle_speed_boost` observer in `behaviors/effects/speed_boost.rs`
- `SpeedBoostTarget` enum in `chips/definition.rs`: variants `Bolt`, `Breaker`, `AllBolts` (Breaker and AllBolts are no-ops for now)

## TriggerChain Unification (refactor/unify-behaviors, as of 2026-03-21) (do not re-flag)

### Core architectural changes
- `ActiveBehaviors` + `ActiveOverclocks` → single `ActiveChains(Vec<TriggerChain>)` resource in `behaviors/active.rs`
- `ConsequenceFired(Consequence)` + `OverclockEffectFired` → single `EffectFired { effect: TriggerChain, bolt: Option<Entity> }` in `behaviors/events.rs`
- `behaviors/consequences/` directory deleted → replaced by `behaviors/effects/` (life_lost, time_penalty, spawn_bolt, shockwave)
- `bolt/behaviors/` directory deleted — all bridges now live in `behaviors/bridges.rs`
- New files: `behaviors/armed.rs` (ArmedTriggers component), `behaviors/evaluate.rs` (TriggerKind + evaluate() fn), `behaviors/events.rs` (EffectFired)
- `ArchetypeDefinition` now has named root fields (`on_bolt_lost`, `on_perfect_bump`, `on_early_bump`, `on_late_bump`: `Option<TriggerChain>`) + `chains: Vec<TriggerChain>` — no more `BehaviorBinding` vec

### New bridge systems in BehaviorSystems::Bridge
- `bridge_cell_impact` — reads `BoltHitCell`, runs `.after(BoltSystems::BreakerCollision)`
- `bridge_breaker_impact` — reads `BoltHitBreaker`, runs `.after(BoltSystems::BreakerCollision)`
- `bridge_wall_impact` — reads `BoltHitWall`, runs `.after(BoltSystems::BreakerCollision)`
- `bridge_cell_destroyed` — reads `CellDestroyed`, unordered (no physics dependency)

### BumpPerformed carries bolt field only
- `BumpPerformed { grade, bolt: Entity }` — no multiplier field; bridge_bump uses bolt to arm specific bolt
- SpeedBoost generalization refactor (refactor/unify-behaviors) removed `multiplier` from BumpPerformed and deleted `apply_bump_velocity` system from bolt domain
- Bump velocity scaling is now handled through `TriggerChain::SpeedBoost { target, multiplier }` fired through `EffectFired`, not through BumpPerformed

### Scenario runner new field
- `ScenarioDefinition.initial_overclocks: Option<Vec<TriggerChain>>` — injects overclock chains at scenario start without going through chip selection UI. Used in `surge_overclock.scenario.ron`.

### Phase 4d status
- 4d is complete on feature/overclock-trigger-chain branch. Plan updated to mark all 4d sub-stages done.

## refactor/unify-behaviors Branch New Content (as of 2026-03-21, do not re-flag)
- `chips/effects/bolt_speed_boost.rs` + `handle_bolt_speed_boost` observer: handles `AmpEffect::SpeedBoost`
- `chips/effects/breaker_speed_boost.rs` + `handle_breaker_speed_boost` observer: handles `AugmentEffect::SpeedBoost`
- `chips/effects/bump_force_boost.rs` + `handle_bump_force_boost` observer: handles `AugmentEffect::BumpForce`
- `chips/effects/tilt_control_boost.rs` + `handle_tilt_control_boost` observer: handles `AugmentEffect::TiltControl`
- `BreakerSpeedBoost`, `BumpForceBoost`, `TiltControlBoost` components in `chips/components.rs` — all already documented in content.md and plugins.md
- `TriggerChain::MultiBolt` and `TriggerChain::Shield` leaf variants: in code and already documented in `docs/design/triggers-and-effects.md` (marked "not yet wired")
- `TriggerKind` / `EvalResult` in `behaviors/evaluate.rs`: internal eval types, not glossary-level terms
- `FrameMutation` / `MutationKind`: added to `docs/design/terminology.md` in 2026-03-21 session

## Spatial/Physics Extraction Architecture (2026-03-24, do not re-flag)

### New crates
- `rantzsoft_spatial2d` — `Position2D`, `Rotation2D`, `Scale2D`, `Global*`, `Velocity2D`, `PreviousVelocity`, `InterpolateTransform2D`, `VisualOffset`, `ApplyVelocity`, `Spatial2D` marker, `DrawLayer` trait, `PositionPropagation`/`RotationPropagation`/`ScalePropagation` enums. Plugin: `RantzSpatial2dPlugin<D: DrawLayer>`.
- `rantzsoft_physics2d` — `Aabb2D` (requires Spatial2D), `CollisionLayers`, `DistanceConstraint`, `CollisionQuadtree` resource, quadtree/CCD math, `RantzPhysics2dPlugin`, `rantzsoft_physics2d::plugin::PhysicsSystems` set (MaintainQuadtree, EnforceDistanceConstraints).
- `rantzsoft_defaults` + `rantzsoft_defaults_derive` — existed before; no game-specific content, re-exports `GameConfig` derive macro.

### What was dissolved
- `physics/` game domain — deleted. Collision systems (`bolt_cell_collision`, `bolt_breaker_collision`, `bolt_lost`, `clamp_bolt_to_playfield`) moved to **bolt domain**.
- `interpolate/` game domain — deleted. Replaced by `rantzsoft_spatial2d` AfterFixedMainLoop propagation pipeline.
- `breaker-derive/` — replaced by `rantzsoft_defaults` + `rantzsoft_defaults_derive`.
- Old `PhysicsSystems::BreakerCollision` and `PhysicsSystems::BoltLost` — now `BoltSystems::BreakerCollision` and `BoltSystems::BoltLost` in `bolt/sets.rs`.

### Scheduling changes
- `FixedFirst`: `save_previous` (spatial2d)
- `FixedUpdate`: `maintain_quadtree.in_set(rantzsoft_physics2d::PhysicsSystems::MaintainQuadtree)`, `apply_velocity` — collision now in bolt domain ordering after MaintainQuadtree
- `AfterFixedMainLoop` (RunFixedMainLoopSystems::AfterFixedMainLoop): `compute_globals → derive_transform → propagate_position → propagate_rotation → propagate_scale` — all chained in rantzsoft_spatial2d

### Chain bolts
- `TriggerChain::ChainBolt { tether_distance }` — now wired; `handle_chain_bolt` observer sends `SpawnChainBolt` message
- `spawn_chain_bolt` system in bolt domain; `break_chain_on_bolt_lost` cleans up on anchor loss
- `DistanceConstraint` component (physics2d) used for tethering; game-level `enforce_distance_constraints` in bolt domain

### Spreading shockwave (not instant)
- `ShockwaveRadius` component grows via `tick_shockwave` each fixed tick
- `shockwave_collision` queries `CollisionQuadtree` each tick for newly-entered cells
- `animate_shockwave` in Update drives visual VFX ring

### Transform is derived, never written
- `Position2D` is canonical — game systems write Position2D, Transform is derived by `derive_transform`
- `DrawLayer` trait + `GameDrawLayer` enum in `shared/` sets Transform.translation.z
- `Aabb2D` has `#[require(Spatial2D)]` — spawning Aabb2D auto-inserts all spatial components

## C7-R Effect Domain Architecture (2026-03-25, do not re-flag)

### behaviors/ → effect/ domain rename
- `behaviors/` domain DELETED. Replaced by `effect/` top-level domain.
- `BehaviorsPlugin` → `EffectPlugin` (in `effect/plugin.rs`)
- `BehaviorSystems::Bridge` → `EffectSystems::Bridge` (in `effect/sets.rs`)
- `ActiveChains(Vec<TriggerChain>)` → `ActiveEffects(Vec<(Option<String>, EffectNode)>)` (in `effect/active.rs`)
- `ArmedTriggers` → `ArmedEffects` (in `effect/armed.rs`)
- `EffectFired { effect: TriggerChain, bolt: Option<Entity> }` DELETED → replaced by typed per-effect events (ShockwaveFired, LoseLifeFired, etc.) dispatched via `fire_typed_event` in `typed_events.rs`
- `behaviors/bridges.rs` → `effect/triggers/` (on_bolt_lost, on_bump, on_no_bump, on_impact, on_death, on_timer)
- `behaviors/effects/` → `effect/effects/` (~20 files, each with `register(app)`)
- `behaviors/evaluate.rs` → `effect/evaluate.rs` (`TriggerKind` deleted — `Trigger` enum used directly)
- `behaviors/events.rs` → `effect/typed_events.rs` (complete redesign)
- `behaviors/active.rs` → `effect/active.rs`
- `behaviors/armed.rs` → `effect/armed.rs`
- `helpers.rs` (new in `effect/`) — shared bridge helpers

### BreakerDefinition / BreakerRegistry moved
- `effect/definition.rs` no longer owns `BreakerDefinition`
- `BreakerDefinition` → `breaker/definition.rs` (canonical; uses `EffectNode` directly)
- `BreakerRegistry` → `breaker/registry.rs` (canonical; re-exported from `effect/`)
- `init_breaker` → `breaker/systems/init_breaker.rs`
- `apply_breaker_config_overrides` → `breaker/systems/init_breaker.rs`

### New triggers added
- `NoBump` — bolt passed breaker without bump attempt; owner: breaker
- `PerfectBumped`, `Bumped`, `EarlyBumped`, `LateBumped` — bolt-perspective post-bump triggers; owner: specific bolt
- `NodeTimerThreshold(f32)` — fires when node timer ratio drops below threshold

### New bridge systems added
- `bridge_no_bump` (in `effect/triggers/on_no_bump.rs`)
- `bridge_cell_death`, `bridge_bolt_death`, `cleanup_destroyed_cells`, `cleanup_destroyed_bolts`, `apply_once_nodes` (in `effect/triggers/on_death.rs`)
- `bridge_timer_threshold` (in `effect/triggers/on_timer.rs`)

### EffectNode/Effect split
- `EffectNode` (the tree): 4 variants: `When`, `Do`, `Until`, `Once` — NO `On` variant
- `Effect` (the leaf enum): ~20 variants covering triggered + passive effects
- `EffectChains` component exists (entity-local chains); `ActiveEffects` resource also exists (global breaker+chip chains)
- `RootEffect` DOES NOT EXIST — was a forward-looking design not implemented
- `ChipDefinition.effects: Vec<EffectNode>` (not Vec<TriggerChain>, not Vec<RootEffect>)
- `BreakerDefinition` has named fields (`on_bolt_lost`, `on_perfect_bump`, `on_early_bump`, `on_late_bump: Option<EffectNode>`) + `chains: Vec<EffectNode>` — no `effects` field

### run/highlights/ sub-domain
- `run/highlights/systems/` holds: `detect_close_save`, `detect_combo_king`, `detect_mass_destruction`, `detect_pinball_wizard`
- Previously at `run/systems/detect_*.rs`; now moved to highlight sub-domain
- `detect_combo_and_pinball.rs` split into `detect_combo_king.rs` + `detect_pinball_wizard.rs`
- `detect_close_save` now orders `.after(BreakerSystems::GradeBump)` (not `.after(BoltSystems::BreakerCollision)`)
- `spawn_highlight_text` IS registered in RunPlugin (Update, PlayingState::Active)
- `HighlightConfig` IS init_resource'd in RunPlugin

### RampingDamage max_bonus removed
- `RampingDamage` now only has `bonus_per_hit: f32` — no `max_bonus` field

### TriggerChain still exists in chips/definition.rs
- `TriggerChain` enum in `chips/definition.rs` is a legacy/parallel chip-side tree (using `On*` wrappers)
- `ChipDefinition.effects: Vec<EffectNode>` (not Vec<TriggerChain>) — chips have migrated to EffectNode
- Do NOT re-flag `TriggerChain` in chips/definition.rs as drift — it may be used by the chip dispatch pipeline

### Three Effect Stores (do not re-flag after 2026-03-25 fix)
- `ActiveEffects` — global Resource, populated by `init_breaker` and `dispatch_chip_effects`. Bridge helpers sweep for global triggers.
- `ArmedEffects` — component on bolt entities. Partially-resolved When trees awaiting deeper trigger.
- `EffectChains` — component on individual entities. Entity-local chains (used for Once/SecondWind-style effects on cells/bolts).
- NOTE: `effect/definition.rs` code comment on `EffectChains` is misleading — says "Replaces both `ActiveEffects` and `ArmedEffects`" but that's wrong; all three types coexist. This is a code comment error (cannot be fixed by docs guard). Documentation has been corrected to accurately describe the three-store model.
- RON chip files use shorthand `OnSelected([...])` syntax (serde alias) rather than `When(trigger: OnSelected, then: [...])` — both are valid; doc examples show both forms intentionally.

## Recurring Drift Patterns
- Stub labels in `plugins.md` folder listing go stale as phases complete
- New system sets added to code without corresponding update to ordering.md defined sets table
- Spawn-coordination messages easily missed since they're internal infrastructure, not gameplay messages
- Intra-domain ordering chains in ordering.md can drift when constraints are restructured
- `PLAN.md` links break when subphase files are moved to `done/` folder — also check parent index.md files (e.g., `phase-2/index.md` had stale subphase links)
- CellTypeDefinition.hp field: always `f32` (not `u32`) — check content.md and data.md on each wave
- `standards.md` scenario runner section: use `cargo scenario` alias (not `dscenario`) for all standard usage; runner is headless by default (`--visual` to open window, no `--headless` flag)
- New chip effect observers land in `chips/effects/` but content.md covers them via the flat component list — don't re-flag observer names as missing unless new component types are added
- Effect domain uses `EffectSystems::Bridge` (not `BehaviorSystems::Bridge`) — check ordering.md and messages.md after any bridge refactor
- `TriggerChain` in chips/definition.rs coexists with `EffectNode` in effect/definition.rs — these are separate types serving different subsystems; don't flag as redundancy
