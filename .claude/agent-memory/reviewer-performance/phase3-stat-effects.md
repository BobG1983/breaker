---
name: Phase 3 stat-effects performance analysis
description: Analysis of 6 recalculate systems added in Phase 3 (feature/stat-effects): entity scale, query patterns, archetype impact, and run_if gap
type: project
---

## Phase 3 Entity Scale

The 6 Active*/Effective* component pairs target bolt and breaker entities only:
- Bolt: 1 normally, up to ~4 with chain-bolt
- Breaker: 1

So each recalculate query matches at most 5 entities in worst case. This is the correct scale for severity calibration.

## How Active* Components Are Pre-Inserted

`fire()` for each stat effect does `world.get_mut::<Active*>(entity)` — push only if component already exists. The component must be pre-inserted at spawn time. Bolt spawn (`spawn_bolt.rs`) and breaker init (`init_breaker_params.rs`) do NOT insert them — they are inserted at chip dispatch time. This means the recalculate queries match ZERO entities until a chip fires. This is significant: 6 queries running over zero entities every FixedUpdate frame — trivial cost now but the `run_if` gap is still worth noting for correctness.

## run_if Gap Confirmed

`EffectPlugin::build()` configures the Recalculate set but attaches NO `run_if` guard. Individual `register()` calls (e.g., `recalculate_damage`) also have no `run_if`. The systems run every FixedUpdate including during pause, ChipSelect state, and any other non-playing game state. Compare: BoltPlugin and BreakerPlugin both use `.run_if(in_state(PlayingState::Active))` on their FixedUpdate systems.

**Why:** Not a real issue at current entity scale (0-5 entities). Queries over 0 entities are near-free. Flag as Minor.

## Unconditional Recalculation: No Change Detection

All 6 recalculate systems run unconditionally every frame — they call `.multiplier()` or `.total()` on every matching entity regardless of whether Active* changed. No `Changed<Active*>` filter. At 1-5 entities, unconditional iteration costs nothing measurable. If entity count grows (Phase 5+), change detection would be the right fix.

**Why:** Minor at current scale. Would become Moderate at 50+ affected entities.

## Vec<f32> Allocation Pattern

Each Active* component stores a `Vec<f32>` (or `Vec<u32>` for piercing). The `.multiplier()` and `.total()` methods iterate this vec inline — no additional allocation. The vec itself is heap-allocated per entity but this is a one-time spawn cost, not per-frame. Clean.

## 6 Systems vs 1: Parallelism Benefit

6 separate systems in the same system set with non-overlapping component access CAN run in parallel in Bevy's scheduler. However: all 6 target different archetypes (ActiveDamageBoosts vs ActiveSpeedBoosts etc.) so even a merged system would not cause contention. At 1-5 entities, the parallelism gain is zero. No action needed.

## Archetype Fragmentation from Active* Components

Each stat type is a separate component. Entities that have some but not all stat effects will occupy different archetypes. With only 2 possible host entities (bolt, breaker) and at most 6 extra components each, fragmentation is bounded and trivial. The key insight: these components are not added/removed at runtime per-frame — they are added once at chip dispatch and persist until reversed. So archetype invalidation only happens at chip selection time, not during gameplay. Clean.

## Intentional Pattern: Silent No-Op Until Chip Fires

The design intent is that recalculate systems produce zero results until at least one chip fires the matching stat. This is correct behavior. The zero-entity case is not a bug.

## Note: recalculate systems are REAL (not placeholder) as of feature/runtime-effects

All 6 recalculate systems (recalculate_speed, recalculate_damage, recalculate_piercing, recalculate_size, recalculate_bump_force, recalculate_quick_stop) are fully implemented — they query (&ActiveXxx, &mut EffectiveXxx) and update the scalar. The "Wave 6 placeholder" comment from earlier phases no longer applies.
