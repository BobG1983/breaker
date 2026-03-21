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

## TriggerChain Unification (refactor/unify-behaviors, as of 2026-03-21) (do not re-flag)

### Core architectural changes
- `ActiveBehaviors` + `ActiveOverclocks` → single `ActiveChains(Vec<TriggerChain>)` resource in `behaviors/active.rs`
- `ConsequenceFired(Consequence)` + `OverclockEffectFired` → single `EffectFired { effect: TriggerChain, bolt: Option<Entity> }` in `behaviors/events.rs`
- `behaviors/consequences/` directory deleted → replaced by `behaviors/effects/` (life_lost, time_penalty, spawn_bolt, shockwave)
- `bolt/behaviors/` directory deleted — all bridges now live in `behaviors/bridges.rs`
- New files: `behaviors/armed.rs` (ArmedTriggers component), `behaviors/evaluate.rs` (TriggerKind + evaluate() fn), `behaviors/events.rs` (EffectFired)
- `ArchetypeDefinition` now has named root fields (`on_bolt_lost`, `on_perfect_bump`, `on_early_bump`, `on_late_bump`: `Option<TriggerChain>`) + `chains: Vec<TriggerChain>` — no more `BehaviorBinding` vec

### New bridge systems in BehaviorSystems::Bridge
- `bridge_cell_impact` — reads `BoltHitCell`, runs `.after(PhysicsSystems::BreakerCollision)`
- `bridge_breaker_impact` — reads `BoltHitBreaker`, runs `.after(PhysicsSystems::BreakerCollision)`
- `bridge_wall_impact` — reads `BoltHitWall`, runs `.after(PhysicsSystems::BreakerCollision)`
- `bridge_cell_destroyed` — reads `CellDestroyed`, unordered (no physics dependency)

### BumpPerformed now carries bolt field
- `BumpPerformed { grade, multiplier, bolt: Entity }` — bolt field added; bridge_bump uses it to arm specific bolt

### Scenario runner new field
- `ScenarioDefinition.initial_overclocks: Option<Vec<TriggerChain>>` — injects overclock chains at scenario start without going through chip selection UI. Used in `surge_overclock.scenario.ron`.

### Phase 4d status
- 4d is complete on feature/overclock-trigger-chain branch. Plan updated to mark all 4d sub-stages done.

## Recurring Drift Patterns
- Stub labels in `plugins.md` folder listing go stale as phases complete
- New system sets added to code without corresponding update to ordering.md defined sets table
- Spawn-coordination messages easily missed since they're internal infrastructure, not gameplay messages
- Intra-domain ordering chains in ordering.md can drift when constraints are restructured
- `PLAN.md` links break when subphase files are moved to `done/` folder — also check parent index.md files (e.g., `phase-2/index.md` had stale subphase links)
- CellTypeDefinition.hp field: always `f32` (not `u32`) — check content.md and data.md on each wave
- `standards.md` scenario runner section: use `cargo scenario` alias (not `dscenario`) for all standard usage; runner is headless by default (`--visual` to open window, no `--headless` flag)
