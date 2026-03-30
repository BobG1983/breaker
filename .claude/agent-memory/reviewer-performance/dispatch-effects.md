---
name: dispatch effects performance analysis
description: dispatch_breaker_effects, dispatch_cell_effects, dispatch_chip_effects, and scenario lifecycle pending effects — scheduling, clone patterns, archetype impact
type: project
---

## Systems and Their Schedules

- `dispatch_breaker_effects` — `OnEnter(GameState::Playing)` (one-shot). Not per-frame. Clean.
- `dispatch_cell_effects` — `OnEnter(GameState::Playing)` (one-shot). Uses `CellEffectsDispatched` marker to prevent double-dispatch on cells. Clean.
- `dispatch_chip_effects` — `Update`, `run_if(in_state(GameState::ChipSelect))`. Event-driven (reads ChipSelected messages). Clean.

## Clone Patterns in Dispatch (One-Shot Context)

### dispatch_breaker_effects/system.rs
- `bound_children.clone()` at line 69: `Vec<(String, EffectNode)>` cloned once per target entity per root effect.
  Target::Cell/AllCells = up to ~50 clones. Target::Breaker = 1 clone. Both one-shot. Acceptable.
- `effect.clone()` at line 62 inside `for entity in target_entities`: clones EffectKind per entity per do_effect per root_effect. One-shot. Acceptable.

### dispatch_cell_effects.rs
- `non_do_children.clone()` at line 132: `Vec<(String, EffectNode)>` cloned once per target entity.
  Worst case: Target::AllCells clones once per cell (up to ~50). One-shot OnEnter. Acceptable.
  NOTE: `for root_effect in effects` is an outer loop — non_do_children and do_children are
  reallocated inside the cell loop for every root_effect on every undispatched cell. At ~50 cells
  with 2-3 root effects each: ~150 Vec allocations total. All one-shot. Acceptable.

### dispatch_chip_effects/system.rs
- `def.effects.clone()` at line 50: clones full effects Vec before inventory borrow.
  Called once per ChipSelected message (episodic). Clean.
- `chip_name.to_owned()` at line 92 and 105: one String per effect dispatch. Episodic. Clean.
- `resolve_target_entities` allocates a Vec<Entity> every call. Called per root_effect per chip.
  At chip-select time (episodic). Acceptable.

## Scenario Lifecycle: apply_pending_*_effects (scenario-runner only)

- `apply_pending_cell_effects` line 1219: `entries.clone()` clones full Vec<(String, EffectNode)>
  once, then `entries_clone` inside the cell loop clones again per cell entity (up to ~50 times).
  Total: 1 + N clones. Runs at most once per scenario run (Local<bool> guard). Acceptable.
- `apply_pending_wall_effects` line 1256: same pattern, ~4 walls. 1 + 4 clones. Acceptable.
- Both systems run every FixedUpdate until `*done = true` — the Local<bool> check is the only cost
  after the first application. Acceptable overhead.

## apply_debug_frame_mutations (scenario-runner FixedUpdate)

- Linear scan of `frame_mutations` every tick until `playing_gate` is true.
  In practice, scenarios have 1-5 frame_mutations entries. O(N) scan is negligible.
- Gated by `playing_gate` run_if. Clean.

## FixedPreUpdate Ungated Systems (scenario-runner)

- `inject_scenario_input`, `apply_perfect_tracking`, `update_force_bump_grade` have no `run_if`.
- Each checks `Option<ResMut<ScenarioInputDriver>>` at the top and early-returns if absent.
- Cost: one resource existence check per tick when driver is absent. Acceptable for test runner.

## Archetype Impact

- `CellEffectsDispatched` marker added to ~50 cell entities on first FixedUpdate tick after
  OnEnter(Playing). Splits cell archetypes: cells without CellEffectsDispatched → cells with it.
  The dispatch query uses `Without<CellEffectsDispatched>` so after dispatch completes, the query
  returns 0 results every subsequent call. Zero overhead after dispatch. Clean design.

## dispatch_wall_effects: One-Shot, Pure No-Op (feature/source-chip-shield-absorption)

- New system: `dispatch_wall_effects` in `wall/systems/dispatch_wall_effects.rs`.
- Registered `OnEnter(GameState::Playing)`, chained after `spawn_walls`. One-shot.
- Body is a no-op: no wall RON definitions currently define effects. Parameters are prefixed `_`.
- Cost: zero. Walls are spawned once; the system has nothing to iterate over. Clean.

## check_chain_arc_count_reasonable: FixedUpdate Checker (feature/source-chip-shield-absorption)

- New invariant checker in scenario runner: `check_chain_arc_count_reasonable`.
- Two queries: `Query<Entity, With<ChainLightningChain>>` + `Query<Entity, With<ChainLightningArc>>`.
- Registered in `checkers_b` chain, which runs under `playing_gate` in FixedUpdate.
  `playing_gate = stats.is_some_and(|s| s.entered_playing)` — gated correctly.
- Both queries return 0 entities in normal play (0-1 chain + 0-1 arc at any time per Phase 5 analysis).
- Per-frame cost at 0 entities: two empty iterator walks + count(). Negligible.
- No allocations. No format! call unless violation fires (branch only taken on actual leak). Clean.

## apply_pending_cell_effects / apply_pending_wall_effects: insert_if_new Pattern (feature/source-chip-shield-absorption)

- Both systems use `commands.entity(entity).insert_if_new((BoundEffects::default(), StagedEffects::default()))`.
- `insert_if_new` is idempotent — no archetype churn on entities that already have the components.
- Both systems fire at most once per scenario run (Local<bool> guard). The insert_if_new is
  called at most ~50 times (cells) or ~4 times (walls) total per scenario lifetime.
- Archetype impact: inserts BoundEffects + StagedEffects on scenario cell/wall entities that
  previously lacked them. Creates new archetypes, but only once per scenario run. Acceptable.

## handle_cell_hit: Shield Absorption (feature/source-chip-shield-absorption)

- `DamageVisualQuery` now includes `Option<&mut ShieldActive>` (changed from `Has<ShieldActive>`
  in the prior branch). Mutable access required for charge decrement and component removal.
- Shield branch: if shield has charges, decrement and `continue` (skip damage). Remove if 0.
- `commands.entity(msg.cell).remove::<ShieldActive>()` called when charges hit 0.
  This is an exceptional path per cell hit, not per-frame. Causes archetype change at that moment,
  but cells only lose their shield once per shield activation.
- The mutable borrow on ShieldActive in DamageVisualQuery means handle_cell_hit cannot run
  concurrently with any other system that reads/writes ShieldActive. In practice no other
  FixedUpdate system accesses ShieldActive. Scheduling: clean, no new parallelism conflicts.

## tag_game_entities: Runs Every FixedUpdate Tick (Ungated)

- `tag_game_entities` is registered in FixedUpdate WITHOUT a `playing_gate` run_if.
- The system runs 4 queries (Bolt/Without<Tag>, Breaker/Without<Tag>, Cell/Without<Tag>, Wall/Without<Tag>).
- After all entities are tagged (typically tick 1), all 4 queries return 0 results every tick.
- Zero-result queries are not free — Bevy still walks the archetype table to determine there are
  no matches. At this entity scale (~50 cells, 1 bolt, 1 breaker, 4 walls) the cost is negligible
  but not literally zero.
- This is CORRECT design (it must catch newly spawned entities mid-run); the cost is acceptable.
- Confirmed as Minor / Watch Later: note if entity counts grow significantly.

## DispatchTargets SystemParam: Four Queries, Event-Gated (Clean)

- `DispatchTargets<'w, 's>` in dispatch_chip_effects/system.rs has 4 queries:
  breakers (With<Breaker>), bolts (With<Bolt>), cells (With<Cell>), walls (With<Wall>).
- All 4 are filtered by marker component. Correct.
- `dispatch_chip_effects` runs in Update, gated by `run_if(in_state(GameState::ChipSelect))`.
  The queries only instantiate when the system runs; outside ChipSelect state there is no cost.
- At chip-select time: each query iterates at most ~50 cells, 1 bolt, 1 breaker, 4 walls.
- `resolve_target_entities` allocates Vec<Entity> per call. Called per root_effect per chip.
  Chip dispatch is episodic. Acceptable.

## dispatch_breaker_effects: Four Separate Top-Level Queries (One-Shot, Clean)

- `dispatch_breaker_effects` takes bolt_query, cell_query, breaker_query, wall_query as
  top-level parameters (not a SystemParam struct). OnEnter(GameState::Playing). One-shot.
- Vec::new() / collect() patterns are all one-shot. Acceptable.

## PushBoundEffects Command: World Access in apply() (Correct Pattern)

- `PushBoundEffects::apply(self, world: &mut World)` uses world.get_entity_mut().
- This is the standard Bevy 0.18 custom Command pattern. Not a hot path.
- `insert_if_new` equivalent done manually (get::<BoundEffects>().is_none() check).
  Bevy's built-in insert_if_new in Commands is deferred; this command needs the check to happen
  at apply time to avoid double-default. Correct and clean.
- Called at most once per entity per dispatch event. Not per-frame.

## apply_pending_cell_effects: entries_clone inside Loop (One-Shot, Bounded)

- `entries.clone()` once before the loop, then `entries_clone` inside the loop (per cell entity).
- Total clones: 1 + N (N = ~50 cells). All one-shot via Local<bool> guard.
- Acceptable at current scale. If cell counts grow to 500+, consider pre-computing
  a single Arc<Vec<...>> to share across entities.

## Summary Verdict

All dispatch systems are either one-shot (OnEnter) or event-driven (message-gated). No
per-frame allocation concerns. Clone patterns are bounded by entity counts and run
at most once per game-session event. The Local<bool> guards in apply_pending_* are efficient
(single bool check per tick). Nothing in this diff introduces per-frame allocation hot paths.
The new shield absorption logic in handle_cell_hit is message-driven and architecturally clean.
dispatch_wall_effects is a pure no-op at current and foreseeable scale.
tag_game_entities running ungated in FixedUpdate is correct design with negligible cost at
current entity scale.
