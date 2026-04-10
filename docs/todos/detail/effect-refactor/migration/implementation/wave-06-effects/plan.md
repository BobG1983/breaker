# Wave 6: Effects + Tick Systems + Conditions (RED → GREEN)

## Specs
- [spec-tests.md](spec-tests.md) — Behavioral test spec
- [spec-code.md](spec-code.md) — Implementation spec

## Goal
Write tests for all 30 effect fire/reverse behaviors, tick systems, and condition evaluation, then implement them.

## RED phase — write failing tests

### Passive effects (8)
- fire pushes to EffectStack, reverse removes, aggregate correct, multiple sources stack

### Spawner effects (12)
- fire spawns entity with correct components + CleanupOnExit, BoltBaseDamage snapshotted

### Tick systems
- Shockwave: expand radius, damage cells, despawn at max
- ChainLightning: state machine (Idle → ArcTraveling → next jump)
- Anchor: plant/unplant state machine
- Attraction: steer bolts
- Pulse: emit periodic shockwaves
- Shield: countdown, reflection cost
- Phantom: countdown, despawn
- TetherBeam: damage cells, cleanup dead endpoints
- GravityWell: pull bolts, despawn expired

### Condition evaluation
- evaluate_conditions fires on false→true, reverses on true→false, no action on no-change
- NodeActive, ShieldActive, ComboActive evaluators

### Other effects
- FlashStep toggle, CircuitBreaker counter+reward, EntropyEngine random, Anchor state machine
- LoseLife sends DamageDealt<Breaker>, Die sends KillYourself<T>, TimePenalty sends ApplyTimePenalty
- RandomEffect delegates to random selection

## RED gate
All tests compile. All tests fail.

## GREEN phase — implement
All fire/reverse impls, all tick systems, evaluate_conditions, condition evaluators.

## GREEN gate
All tests pass. Do NOT modify tests.

## Parallelism
Passive, spawner, tick, condition, message-based — all independent groups.

## Docs to read
- `effect-refactor/migration/new-effect-implementations/` — all 30 per-effect behavioral specs
- `effect-refactor/creating-effects/effect-api/` — fireable.md, reversible.md, passive-effect.md
- `effect-refactor/evaluating-conditions/` — evaluate-conditions.md, is-node-active.md, is-shield-active.md, is-combo-active.md
- `effect-refactor/rust-types/components/` — component definitions for spawned entities
