---
name: effect-fired-pipeline
description: STALE — EffectFired was deleted in C7-R (2026-03-25). Replaced by per-effect typed events in effect/typed_events.rs. Do not use this file.
type: reference
---

> **RETIRED 2026-03-25**: `EffectFired` was deleted in the C7-R refactor. The behaviors/ domain was renamed to effect/, and the unified EffectFired event was replaced by per-effect typed events (ShockwaveFired, LoseLifeFired, SpeedBoostFired, etc.) dispatched via fire_typed_event() in effect/typed_events.rs. See planner-spec/domain-inventory.md effect domain section for current state. The content below is historical only.

# EffectFired Pipeline

## Emission
- 7 bridge systems in FixedUpdate (bridge_bolt_lost, bridge_bump, bridge_bump_whiff, bridge_cell_impact, bridge_breaker_impact, bridge_wall_impact, bridge_cell_destroyed) read domain messages and evaluate ActiveChains + ArmedTriggers via evaluate() -> EvalResult::Fire -> commands.trigger(EffectFired)
- ActiveChains populated at init from archetype RON + at runtime from handle_overclock (chip selection)
- ArmedTriggers component on bolt entities for multi-step chains (Arm -> later Fire)

## Observer Pattern
- All handlers registered via .add_observer() in BehaviorsPlugin::build
- Signature: `fn handler(trigger: On<EffectFired>, ...query params...)`
- Self-selection: `let TriggerChain::Variant { fields } = &trigger.event().effect else { return; };`
- Every observer receives every EffectFired; non-matching variants return immediately

## Existing Handlers (6)
1. handle_shockwave -> spawns ShockwaveRadius entity
2. handle_spawn_bolt -> writes SpawnAdditionalBolt message
3. handle_chain_bolt -> writes SpawnChainBolt message (needs bolt entity)
4. handle_speed_boost -> mutates bolt Velocity2D
5. handle_life_lost -> decrements LivesCount, writes RunLost at zero
6. handle_time_penalty -> writes ApplyTimePenalty message

## Unhandled Leaf Variants
- MultiBolt { base_count, count_per_level, stacks } -- no handler, no consumer
- Shield { base_duration, duration_per_level, stacks } -- no handler, no consumer

## Registration
- .add_observer(handler_fn) in BehaviorsPlugin::build
- effects/mod.rs declares sub-modules
- plugin.rs imports from effects::module::handler_fn
