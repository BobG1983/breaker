# Spec Revision Plan

All 18 specs written, all 17 reviews complete. This document tracks the specific revisions needed before launching writer-tests/writer-code.

## Status
- Specs written: 18/18 (9 test + 9 code, wave 8 has no code spec)
- Reviews complete: 17/17
- Revisions applied: 0/17

## Class 1: Prerequisites (ALL specs)
Every spec needs an explicit prerequisites section stating which waves must be complete before it can compile. This is inherent in clean-room — not a spec error, just needs to be explicit.

## Class 2: Position/Spatial (waves 9-12 code specs)
- `Transform` → `Position2D` / `GlobalPosition2D` (entities use rantzsoft_spatial2d, not Bevy Transform)
- `SpatialIndex` → doesn't exist. Use `commands.entity(e).remove::<Aabb2D>()` — quadtree auto-deregisters via maintain_quadtree's RemovedComponents<Aabb2D> handler
- Affects: wave 9 code, wave 10 code, wave 11 code, wave 12 code

## Class 3: File path alignment (ALL specs)
- Test spec "Tests go in" must match code spec file paths
- Use directory module layout per project convention (mod.rs + system.rs + tests.rs)
- Affects: every spec pair

## Class 4: Same-frame deduplication (waves 9-12)
- Kill handlers need local `HashSet<Entity>` for same-frame dedup
- `commands.insert(Dead)` is deferred — `Without<Dead>` won't catch duplicates within one system run
- Affects: all kill handler specs

## Class 5: Wave-specific fixes

### Wave 4 (functions)
- [ ] Fix Constraints file paths (core/types → stacking/dispatch)
- [ ] Add During/Until test behaviors (apply_scoped_tree, reverse_scoped_tree)
- [ ] Fix ComboStreak resource name → `ComboStreak { count: u32 }`
- [ ] Fix naming: `reversible_to_effect_type` consistently
- [ ] Add On section note: Terminal::Fire not Tree::Fire
- [ ] Add missing behaviors: route_effect During, condition_active Some(true) skip, stamp Once, 3-entry aggregate, mixed staged
- [ ] Add EffectStack<T> generic Component bound note (`T: 'static`)
- [ ] Clarify is_shield_active Bevy API (may need &mut World)

### Wave 5 (triggers)
- [ ] Remove `.after(tick_effect_timers)` from on_time_expires register (Bridge before Tick by set ordering)
- [ ] Remove `.after(check_node_timer_thresholds)` from threshold bridge register
- [ ] Split on_no_bump_occurred into separate registration with both .after() constraints
- [ ] Fix EffectTimerExpired field: `duration` → `original_duration`
- [ ] Add `Res<NodeTimer>` as explicit parameter for check_node_timer_thresholds
- [ ] Fix file paths: watch_spawn_registry, track_combo_streak
- [ ] Fix behavior 58 Given (message not EffectTimers state)
- [ ] Add missing behaviors: tick sends original_duration, bridge reads from message
- [ ] Resolve NodeTimerThresholdCrossed.ratio type (f32 vs OrderedFloat)

### Wave 6 (effects)
- [ ] Remove tick_effect_timers (G15) — wave 5 scope
- [ ] Add dispatch test section (fire_dispatch 30 arms, reverse_dispatch 16 arms)
- [ ] Fix AnchorActive: add `source: String` field for tick_anchor to use
- [ ] Fix sync_shockwave_visual mapping: specify Scale2D = ShockwaveRadius (diameter)
- [ ] Add despawned-entity guard tests
- [ ] Fix test file paths to match code spec
- [ ] Fix OrderedFloat inconsistency in spawner configs (spawners use plain f32 per design docs)
- [ ] Fix EntropyEngine: does NOT have separate reset system (resets on fire per system-sets.md)

### Wave 7 (death pipeline)
- [ ] Add EffectSystems::Tick to prerequisites
- [ ] Fix apply_damage_to_targets helper signature (remove invalid `impl QueryFilter`)
- [ ] Resolve RequiredToClear: add scope deferral note (wave 9 handles it)
- [ ] Resolve Changed<Hp>: explicitly state it's omitted, Without<Dead> is the filter
- [ ] Add src/shared/systems/ module creation to wiring
- [ ] Fix game.rs plugin wiring to use PluginGroupBuilder .add() pattern
- [ ] Align Hp/KilledBy/Dead file paths between test and code specs

### Wave 9 (cell domain)
- [ ] Fix Locked cell filtering: collision sends DamageDealt, apply_damage drops (behavior 9)
- [ ] Fix system name: cell_damage_feedback → cell_damage_visual
- [ ] Fix Sprite → MeshMaterial2d<ColorMaterial> + ResMut<Assets<ColorMaterial>>
- [ ] Fix SpatialIndex → remove::<Aabb2D>() pattern
- [ ] Fix Transform → Position2D
- [ ] Add same-frame dedup (local HashSet) to kill handler
- [ ] Expand wiring: prelude, node plugin, guardian cells, bolt_cell_collision pierce query
- [ ] Resolve and remove Open Questions section
- [ ] Fix behavior 21 self-correction prose
- [ ] Narrow behaviors 24-25 to cell domain outputs only
- [ ] Add hp.starting=0.0 edge case

### Wave 10 (bolt domain) — MOST CRITICAL
- [ ] **ExtraBolt distinction**: bolt_lost only kills extra bolts, baseline bolts respawn
- [ ] Fix behavior 3 timer semantics (Without<Birthing> = no tick, not suppressed tick)
- [ ] Add Without<Dead> filter tests for tick_bolt_lifespan and bolt_lost
- [ ] Fix GlobalPosition2D test setup
- [ ] Remove compile-time guarantee behaviors (6, 12, 37)
- [ ] Fix same-frame dedup in kill handler
- [ ] Remove Sections E/F (wave 7 scope)
- [ ] Fix BoltLostWriters SystemParam migration
- [ ] Fix prelude/messages.rs, bolt_lost/tests/helpers.rs updates
- [ ] Remove "BEFORE EffectSystems::Bridge" ordering constraint

### Wave 11 (wall domain)
- [ ] Fix Transform → Position2D
- [ ] Fix SpatialIndex → remove::<Aabb2D>()
- [ ] Resolve shield/second-wind scope: add Hp but timer-expiry stays direct despawn for now
- [ ] Fix test file path to directory module layout
- [ ] Remove behavior 9 (contradicts "Do NOT test apply_damage" constraint)
- [ ] Add Fire(Die) path behavior (wall without Hp killed via Die)
- [ ] Add nonexistent victim behavior
- [ ] Fix builder ambiguity (callers insert Hp, not builder method)

### Wave 12 (breaker domain) 
- [ ] **Remove DespawnEntity** from kill handler (breaker persists)
- [ ] Fix system name → handle_breaker_kill
- [ ] Remove Sections A & B (wave 7 scope)
- [ ] Add LivesCount removal note
- [ ] Add missing edge cases (killer gone, stale victim)
- [ ] Point to concrete builder file (terminal.rs build_core)

## Approach
For each wave: send revision instructions to spec writers, they update specs in-place, then re-review.
