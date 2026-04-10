# Wave 4: Non-System Functions (RED → GREEN)

## Specs
- [spec-tests.md](spec-tests.md) — Behavioral test spec
- [spec-code.md](spec-code.md) — Implementation spec

## Goal
Write tests for all non-system functions, then implement them.

## RED phase — write failing tests

### EffectStack<T>
- push adds entry, remove removes matching (source, config), aggregate computes correctly
- Remove with no match does nothing
- Multiple sources, ordering

### Walking algorithm
- walk_effects processes staged before bound
- Trigger matching (exact equality, no wildcards)
- When: matching trigger evaluates inner tree, non-matching skips
- When: inner trigger gate is always armed to staged (even same trigger — "bumped twice")
- Once: matching trigger evaluates + queues removal
- Sequence: evaluates children left to right
- On: resolves participant from context, mismatched context skips
- Route: queues route_effect command

### Fire/reverse dispatch
- Each EffectType variant dispatches to correct config.fire()
- Each ReversibleEffectType variant dispatches to correct config.reverse()

### Command extensions
- stamp_effect appends to BoundEffects (always, even duplicates)
- route_effect(Bound) appends to BoundEffects, route_effect(Staged) appends to StagedEffects
- stage_effect appends to StagedEffects
- remove_effect(Bound/Staged) removes matching entry, no-match does nothing
- reverse_effect no-match does nothing

### Passive effects (per-config)
- fire pushes to EffectStack, reverse removes
- aggregate computes correct value (product for multipliers, sum for additive)

## RED gate
All tests compile. All tests fail (functions are `todo!()`).

## GREEN phase — implement

- EffectStack push/remove/aggregate
- walk_effects + per-node evaluators (fire, when with arming, once, during, until, sequence, on, route)
- Fire/reverse dispatch match arms (30 + 16)
- All Command::apply() implementations
- Condition evaluators (is_node_active, is_shield_active, is_combo_active)
- All passive effect trait impls (fire/reverse with EffectStack, aggregate)

## GREEN gate
All tests pass. Do NOT modify tests.

## Notes
- Complex effect fire() implementations that spawn entities work here, but spawned entities won't tick yet (tick systems come in wave 5).

## Docs to read
- `effect-refactor/walking-effects/` — walking-algorithm.md, when.md, once.md, until.md, sequence.md, on.md, route.md, fire.md, arming-effects.md
- `effect-refactor/command-extensions/` — all command extension specs
- `effect-refactor/creating-effects/effect-api/` — fireable.md, reversible.md, passive-effect.md
- `effect-refactor/rust-types/effect-stacking/` — EffectStack, PassiveEffect
- `effect-refactor/storing-effects/` — BoundEffects, StagedEffects
- `effect-refactor/creating-effects/wiring-an-effect.md` — dispatch match arm pattern
