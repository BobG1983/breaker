# Spec Revision Plan

All 18 specs written, all 17 reviews complete. This document tracks revisions needed.

## Global Rules for ALL Revision Agents

Every revision agent prompt MUST include:

```
## CLEAN ROOM RULES
1. Do NOT reference src/ code. Source of truth is docs/todos/detail/ ONLY.
2. IF YOU REFERENCE ANYTHING UNDER src/ YOUR WORK WILL BE DELETED AND REDONE.
3. Assume the following types/systems already exist from earlier waves — do NOT flag their absence:
   - All types from wave 2 scaffold (Tree, ScopedTree, Trigger, TriggerContext, EffectType, 
     BoundEffects, StagedEffects, all config structs, all traits, all death pipeline types)
   - All systems from earlier waves (walk_effects, fire_dispatch, apply_damage, detect_deaths, etc.)
4. Add a Prerequisites section listing which waves must be complete, but do NOT treat 
   missing code as a spec error.
```

## Status
- Specs written: 18/18
- Reviews complete: 17/17
- Revisions applied: 4/17 (waves 4 test, 10 both, 12 both)
- Revisions still needed: 13 (waves 4 code, 5 both, 6 both, 7 both, 8 test, 9 both, 11 both)

## User Corrections (override reviewer findings)

### All bolts treated the same through death pipeline
bolt_lost sends KillYourself<Bolt> for ALL lost bolts — no ExtraBolt distinction. If too punishing later, BoltLost gains a Primary/Extra identifier and the effect system decides whether to reduce lives. For now: all bolts die the same way.

**Wave 10 needs re-revision**: undo the ExtraBolt distinction that was added. Remove behavior 10 (baseline bolt not killed) and behavior 11 (BoltLost for baseline). Make all bolt_lost behaviors apply to all bolts uniformly.

### Without<Dead> IS sufficient for dedup
Each entity only gets one KillYourself per frame because detect systems query different marker components (With<Cell>, With<Bolt>, etc.). Dead is inserted by the kill handler, visible next frame. No HashSet needed.

**Remove HashSet dedup from ALL kill handler specs** (waves 9, 10, 11, 12 code specs).

## Class 1: Clean-room context (ALL specs)
Add prerequisites section to every spec. Use the global rules above. NOT a spec rewrite — just add context.

## Class 2: Position/Spatial (waves 9-12 code specs)
- `Transform` → `Position2D` / `GlobalPosition2D`
- `SpatialIndex` → `commands.entity(e).remove::<Aabb2D>()` (quadtree auto-deregisters)

## Class 3: File path alignment (ALL specs)
- Align test spec "Tests go in" with code spec file paths
- Use directory module layout per project convention

## Wave-specific fixes

### Wave 4 (functions) — test spec DONE, code spec needs revision
- [x] Fix Constraints file paths
- [x] Add During/Until test behaviors
- [x] Fix ComboStreak resource name
- [x] Fix naming consistency
- [x] Add On section note
- [x] Add missing behaviors
- [x] Add prerequisites
- [ ] Code spec: fix EffectStack<T> generic Component bound
- [ ] Code spec: clarify is_shield_active Bevy API
- [ ] Code spec: fix naming inconsistency
- [ ] Code spec: add ComboStreak name
- [ ] Code spec: add On-context-separation note

### Wave 5 (triggers) — revision IN FLIGHT
- [ ] Remove .after(tick_effect_timers) from on_time_expires
- [ ] Remove .after(check_node_timer_thresholds) from threshold bridge
- [ ] Split on_no_bump_occurred registration
- [ ] Fix EffectTimerExpired field name
- [ ] Add Res<NodeTimer> parameter
- [ ] Fix file paths
- [ ] Fix behavior 58 Given
- [ ] Add missing behaviors
- [ ] Resolve NodeTimerThresholdCrossed.ratio type

### Wave 6 (effects)
- [ ] Remove tick_effect_timers (G15) — wave 5 scope
- [ ] Add dispatch test section
- [ ] Fix AnchorActive: add `source: String` field
- [ ] Fix sync_shockwave_visual mapping
- [ ] Add despawned-entity guard tests
- [ ] Fix test file paths
- [ ] Fix OrderedFloat inconsistency in spawner configs
- [ ] Fix EntropyEngine reset (no separate system)

### Wave 7 (death pipeline)
- [ ] Add EffectSystems::Tick to prerequisites
- [ ] Fix apply_damage helper signature
- [ ] Resolve RequiredToClear (scope deferral to wave 9)
- [ ] State Changed<Hp> is omitted deliberately
- [ ] Add src/shared/systems/ module creation to wiring
- [ ] Fix game.rs plugin wiring pattern
- [ ] Align component file paths

### Wave 8 (integration tests)
- [ ] Fix behavior 10 garbled text
- [ ] Fix behavior 11 cascade test mechanism
- [ ] Scope behaviors 9/10 to testable assertions (or add stub kill handlers)
- [ ] Fix BumpPerformed missing breaker field
- [ ] Fix behavior 5 concrete NodeState values

### Wave 9 (cell domain)
- [ ] Fix Locked cell filtering (pipeline drops, not collision)
- [ ] Fix system name → cell_damage_visual
- [ ] Fix Sprite → MeshMaterial2d<ColorMaterial>
- [ ] Fix SpatialIndex → remove::<Aabb2D>()
- [ ] Fix Transform → Position2D
- [ ] Remove HashSet dedup (Without<Dead> sufficient)
- [ ] Expand wiring section
- [ ] Remove Open Questions section
- [ ] Fix behavior 21 prose
- [ ] Narrow behaviors 24-25 to cell domain outputs
- [ ] Add hp.starting=0.0 edge case

### Wave 10 (bolt domain) — REVISED but needs re-revision
- [x] ~~ExtraBolt distinction~~ → UNDO: all bolts treated the same
- [x] Fix timer semantics
- [x] Add Without<Dead> filter tests
- [x] Fix GlobalPosition2D setup
- [x] Remove compile-time guarantee behaviors
- [x] Remove wave-7 scope sections
- [x] Fix BoltLostWriters migration
- [x] Fix prelude updates
- [x] Remove ordering constraint
- [ ] UNDO ExtraBolt distinction — all bolts go through death pipeline uniformly
- [ ] Remove HashSet dedup (Without<Dead> sufficient)

### Wave 11 (wall domain)
- [ ] Fix Transform → Position2D
- [ ] Fix SpatialIndex → remove::<Aabb2D>()
- [ ] Resolve shield/second-wind scope
- [ ] Fix test file path
- [ ] Remove behavior 9 (contradicts constraint)
- [ ] Add Fire(Die) path behavior
- [ ] Add nonexistent victim behavior
- [ ] Fix builder ambiguity
- [ ] Remove HashSet dedup (Without<Dead> sufficient)

### Wave 12 (breaker domain) — DONE
- [x] Remove DespawnEntity
- [x] Fix system name
- [x] Remove Sections A & B
- [x] Add LivesCount removal
- [x] Add missing edge cases
- [x] Point to concrete builder file
- [ ] Remove HashSet dedup if present (Without<Dead> sufficient)
