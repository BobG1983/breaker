---
name: effect-fired-pipeline
description: End-to-end EffectFired data flow -- emission (bridges + armed triggers), observer handler pattern, TriggerChain leaf variants, registration, and unhandled variants (MultiBolt, Shield)
type: reference
---

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
