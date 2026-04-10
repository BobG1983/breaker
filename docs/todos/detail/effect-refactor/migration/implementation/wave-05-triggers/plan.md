# Wave 5: Trigger Bridge Systems (RED → GREEN)

## Specs
- [spec-tests.md](spec-tests.md) — Behavioral test spec
- [spec-code.md](spec-code.md) — Implementation spec

## Goal
Write tests for all trigger bridge systems and game systems, then implement them.

## RED phase — write failing tests

### Bump bridges (10 systems)
- Each reads correct message type and filters by grade
- Each builds correct TriggerContext::Bump { bolt, breaker }
- Local bridges walk only bolt + breaker; globals walk all entities
- NoBumpOccurred filters on BumpStatus::Inactive
- bolt: None skips bolt walk but still walks breaker

### Impact bridges (2 × 6)
- Each sub-bridge reads correct collision message
- Impacted walks both participants; ImpactOccurred walks all entities

### Death bridges (4 monomorphized)
- Reads Destroyed<T>, dispatches Died → Killed → DeathOccurred in order
- Environmental death (killer=None) skips Killed
- Reads from previous frame (message persistence)

### Bolt lost bridge
- Reads BoltLost { bolt, breaker }, walks all entities

### Node bridges
- on_node_start_occurred fires on OnEnter(Playing)
- on_node_end_occurred fires on OnExit(Playing)
- on_node_timer_threshold_occurred reads NodeTimerThresholdCrossed

### Time bridge
- on_time_expires reads EffectTimerExpired, dispatches TimeExpires(duration)

### Game systems
- tick_effect_timers decrements timers, sends EffectTimerExpired, removes expired
- check_node_timer_thresholds fires on crossing, tracks fired set
- track_combo_streak increments on perfect, resets on other
- watch_spawn_registry stamps on Added<Bolt/Cell/Wall/Breaker>
- reset_node_timer_thresholds clears fired set

## RED gate
All tests compile. All tests fail.

## GREEN phase — implement
All bridge systems and game systems.

## GREEN gate
All tests pass. Do NOT modify tests.

## Parallelism
Each trigger category is independent — bump, impact, death, bolt-lost, node, time can be written/implemented in parallel.

## Docs to read
- `effect-refactor/migration/new-trigger-implementations/bump/` — all 10 bridge specs + types.md
- `effect-refactor/migration/new-trigger-implementations/impact/` — on_impacted.md, on_impact_occurred.md
- `effect-refactor/migration/new-trigger-implementations/death/` — on_destroyed_*.md
- `effect-refactor/migration/new-trigger-implementations/bolt-lost/` — on_bolt_lost_occurred.md, types.md
- `effect-refactor/migration/new-trigger-implementations/node/` — on_node_*.md, check_node_timer_thresholds.md
- `effect-refactor/migration/new-trigger-implementations/time/` — on_time_expires.md, tick_effect_timers.md
- `effect-refactor/dispatching-triggers/dispatch-algorithm.md` — context population table
- `effect-refactor/rust-types/trigger-context.md`
- `effect-refactor/storing-effects/spawn-stamp-registry.md`
- `effect-refactor/rust-types/resources/combo-streak.md`
- `effect-refactor/rust-types/resources/node-timer-threshold-registry.md`
