# Adding a New Effect

Step-by-step reference for adding an effect to the new effect system.

## 1. Create per-effect module

Create `effect/effects/<name>/` (or `effect/effects/<name>.rs` for simple effects).

Implement:
- `pub(crate) fn fire(entity: Entity, config: &Config, source: &SourceId, context: &TriggerContext, world: &mut World)` — execute the effect on the entity
- `pub(crate) fn reverse(entity: Entity, config: &Config, source: &SourceId, world: &mut World)` — undo the effect (only if reversible)
- Per-effect components and runtime systems (if any)

For **simple effects** (single scalar parameter), use a bare type: `SpeedBoost(f32)`. For **complex effects** (multiple fields), create a config struct: `ShockwaveConfig { base_range: f32, speed: f32, ... }`.

## 2. Add variant to EffectType

In `effect/core/types/definitions/enums.rs`:
```rust
enum EffectType {
    // ...
    NewEffect(NewEffectConfig),
}
```

If the effect is **reversible** (can be undone — stat boosts, toggles, etc.), also add to `ReversibleEffectType`:
```rust
enum ReversibleEffectType {
    // ...
    NewEffect(NewEffectConfig),
}
```

## 3. Add match arms in dispatch

In `dispatch_fire()`:
```rust
EffectType::NewEffect(cfg) => new_effect::fire(entity, cfg, source, context, world),
```

In `dispatch_reverse()` (if reversible):
```rust
ReversibleEffectType::NewEffect(cfg) => new_effect::reverse(entity, cfg, source, world),
```

The compiler enforces exhaustive matching — missing arms won't compile.

## 4. Write fire() and reverse()

In the per-effect module:

- `fire()` reads entity components from `World`, spawns/mutates as needed
- `reverse()` undoes what `fire()` did (e.g., remove a speed multiplier from `ActiveSpeedBoosts`)
- Effects that spawn entities (Shockwave, ChainLightning, Explode) use `world.spawn()`

## 5. Write behavioral spec

Create `docs/todos/detail/effect-desugaring-node-running-trigger/effects/<name>.md` documenting config struct, fire behavior, reverse behavior (if reversible), components, and messages.

## 6. Runtime systems (if needed)

Some effects need tick systems (damage application, lifetime expiry, movement). Register them in the effect plugin via `new_effect::register(app)` called from `effect/effects/mod.rs`.

## 7. RON syntax

Config struct variants:
```ron
Fire(NewEffect(field_a: 1.0, field_b: 2.0))
```

Bare type variants:
```ron
Fire(SpeedBoost(1.5))
```

## Additional concerns

### GameEntity trait

Effects that deal damage send `DamageDealt<T>` where `T: GameEntity`. The `GameEntity` trait is implemented on `Bolt`, `Cell`, `Wall`, `Breaker`. Damage-dealing effects receive a `TriggerContext` to propagate kill attribution through the damage chain.

### EffectSourceChip

Spawned effect entities (shockwave ring, explosion request, chain lightning chain, etc.) carry an `EffectSourceChip` component for deferred chip attribution. Damage-application systems read this to populate `DamageDealt<T>.source_chip`. Set it from the `source: &SourceId` parameter in `fire()`.

### Command extensions

Effects that need world access at dispatch time execute inside `EntityCommand::apply` (via `fire_effect` command extension). The `fire()` function already receives `&mut World`. For effects that need to queue further commands (e.g., meta-effects like CircuitBreaker firing sub-effects), call `dispatch_fire` directly with an incremented depth counter on the `TriggerContext`.

### Reversibility rules

- Direct `During(condition, Fire(X))` and `Until(trigger, Fire(X))` require X to be reversible
- Nested `During(condition, When(trigger, Fire(X)))` allows any effect (the listener is reversed, not the effect)
- The builder enforces this at compile time; the RON loader validates at load time
