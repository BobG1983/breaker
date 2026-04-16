# Adding a New Effect

Step-by-step reference for adding an effect to the effect system.

The pattern is: **a config struct in `effect_v3/effects/<name>/` that implements `Fireable` (and `Reversible` if applicable), plus arms in the dispatch matches and the `EffectType` enum.** The compiler enforces exhaustiveness — every match arm you forget will be a compile error.

## 1. Create the per-effect module

Pick the right shape for the effect's complexity:

- **Trivial** (single `config.rs` file, no runtime systems, no extra components):
  ```
  effects/my_effect/
    mod.rs       — pub re-exports + register() (typically just impl Fireable's default register)
    config.rs    — MyEffectConfig + impl Fireable + impl Reversible (if reversible)
  ```
- **With runtime systems**:
  ```
  effects/my_effect/
    mod.rs
    config.rs
    components.rs    — per-effect components (markers, state)
    systems.rs       — tick/cleanup systems + tests, OR
    systems/
      mod.rs
      system.rs
      tests.rs
  ```
- **With nested config tests**:
  ```
  effects/my_effect/
    mod.rs
    config/
      mod.rs
      config_impl.rs   — MyEffectConfig + trait impls
      tests.rs
    components.rs
    systems/
      mod.rs
      system.rs
      tests/
  ```

Look at `effect_v3/effects/speed_boost/` (trivial), `effect_v3/effects/pulse/` (with runtime), or `effect_v3/effects/shield/` (with config tests) for templates.

## 2. Define the config struct

In `config.rs` (or `config/config_impl.rs`):

```rust
use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MyEffectConfig {
    pub multiplier: OrderedFloat<f32>,
    // ... other fields
}
```

Required derives: `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`.

Float fields **must** be `OrderedFloat<f32>` (not bare `f32`) so the enclosing `EffectType` enum can derive `Hash` and `Eq` automatically. This propagates: any other type used in the config that contains an `f32` also needs to use `OrderedFloat`.

## 3. Implement Fireable

Every effect implements `Fireable`:

```rust
use crate::effect_v3::traits::Fireable;

impl Fireable for MyEffectConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        // Read components, mutate, spawn child entities, send messages, etc.
        // `source` is the chip name (or empty string for non-chip-sourced).
        // `world` is exclusive — query whatever you need.
    }

    // Optional: override the default no-op register if the effect
    // has tick systems, cleanup systems, or per-node reset systems.
    fn register(app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (tick_my_effect, cleanup_my_effect)
                .in_set(EffectV3Systems::Tick),
        );
    }
}
```

The body of `fire` is up to the effect. Common patterns:

- **Stack-based passive**: push onto an `EffectStack<MyEffectConfig>` component on the entity. Recalculation happens in a `Tick` system.
- **Component-inserting**: `world.entity_mut(entity).insert(MyEffectActive { ... })`.
- **Spawned entity**: `world.spawn((MyEffectMarker, EffectSourceChip::from_source(source), ...))`.
- **Message-sending**: send a Bevy message that another domain consumes (e.g. `LoseLife` sends `LoseLifeRequest`).

`source` is the originating chip name. Pass it through to `EffectSourceChip::from_source(source)` on any spawned effect entity that will eventually deal damage, so damage attribution can find its way back to the chip.

## 4. Implement Reversible (if applicable)

If the effect can be cleanly undone, implement `Reversible`. The trait extends `Fireable`:

```rust
use crate::effect_v3::traits::Reversible;

impl Reversible for MyEffectConfig {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World) {
        // Undo what fire did. For stack passives: remove the matching entry.
        // For singletons: remove the marker / despawn child.
    }

    // Override only if the effect is stack-based:
    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<EffectStack<MyEffectConfig>>(entity) {
            stack.retain_by_source(source);
        }
    }
}
```

The default `reverse_all_by_source` calls `reverse` once. Override it for stack-based passives so that "remove every instance from this source" is a single call.

## 5. Add the variant to EffectType

In `effect_v3/types/effect_type.rs`:

```rust
pub enum EffectType {
    // ... existing variants
    MyEffect(MyEffectConfig),
}
```

The compiler will now require you to add an arm to `fire_dispatch_does_not_panic_for_any_effect_type_variant` in `effect_v3/dispatch/fire_dispatch.rs`. Update the variant count assertion:

```rust
assert_eq!(types.len(), 31, "update all_effect_types when EffectType gains variants");
```

Also add a representative instance to `passive_effect_types()` or `active_effect_types()` in that test so the smoke test fires the new variant.

## 6. Add the dispatch arm

In `effect_v3/dispatch/fire_dispatch.rs`:

```rust
pub fn fire_dispatch(effect: &EffectType, entity: Entity, source: &str, world: &mut World) {
    match effect {
        // ... existing arms
        EffectType::MyEffect(config) => config.fire(entity, source, world),
    }
}
```

The match is exhaustive — Rust will refuse to compile until you add the arm.

## 7. (If reversible) Add to ReversibleEffectType and reverse dispatch

In `effect_v3/types/reversible_effect_type.rs`:

```rust
pub enum ReversibleEffectType {
    // ... existing variants
    MyEffect(MyEffectConfig),
}
```

Add the conversion arms:

```rust
impl From<ReversibleEffectType> for EffectType {
    fn from(reversible: ReversibleEffectType) -> Self {
        match reversible {
            // ...
            ReversibleEffectType::MyEffect(c) => Self::MyEffect(c),
        }
    }
}

impl TryFrom<EffectType> for ReversibleEffectType {
    type Error = ();
    fn try_from(effect: EffectType) -> Result<Self, Self::Error> {
        match effect {
            // ...
            EffectType::MyEffect(c) => Ok(Self::MyEffect(c)),
            _ => Err(()),
        }
    }
}
```

Then in `effect_v3/dispatch/reverse_dispatch/system.rs`, add arms to all three dispatch functions:

```rust
pub fn reverse_dispatch(effect: &ReversibleEffectType, entity: Entity, source: &str, world: &mut World) {
    match effect {
        // ...
        ReversibleEffectType::MyEffect(config) => config.reverse(entity, source, world),
    }
}

pub fn fire_reversible_dispatch(effect: &ReversibleEffectType, entity: Entity, source: &str, world: &mut World) {
    match effect {
        // ...
        ReversibleEffectType::MyEffect(config) => config.fire(entity, source, world),
    }
}

pub fn reverse_all_by_source_dispatch(effect: &ReversibleEffectType, entity: Entity, source: &str, world: &mut World) {
    match effect {
        // ...
        ReversibleEffectType::MyEffect(config) => config.reverse_all_by_source(entity, source, world),
    }
}
```

All three matches are exhaustive over `ReversibleEffectType`, so the compiler will catch any missing arm.

## 8. Register the config in EffectV3Plugin

In `effect_v3/plugin.rs`, add a `register` call alongside the others:

```rust
effects::AnchorConfig::register(app);
effects::AttractionConfig::register(app);
// ...
effects::MyEffectConfig::register(app);
```

Add the call **even if** `register` is the default no-op. This is the "no silently dropped systems" guarantee — adding tick systems later won't be forgotten because the call already exists.

## 9. Re-export from effects/mod.rs

In `effect_v3/effects/mod.rs`:

```rust
pub mod my_effect;
pub use my_effect::MyEffectConfig;
```

## 10. RON syntax

Once the variant is in `EffectType`, RON deserialization works automatically thanks to serde:

```ron
Fire(MyEffect(multiplier: 1.5))
```

Or inside a tree:

```ron
On(StampTarget::Bolt, Fire(MyEffect(multiplier: 1.5)))
When(PerfectBumpOccurred, Fire(MyEffect(multiplier: 2.0)))
```

## Things you do **not** need to do

- **No need to register the effect with the walker.** The walker is generic — it dispatches via `EffectType`, which now contains your variant.
- **No need to add a special case to `EffectCommandsExt`.** The existing `commands.fire_effect` accepts any `EffectType`.
- **No need to update the trigger system.** Triggers are decoupled from effects — any trigger can fire any effect.
- **No need to write a behavioral spec by hand.** The `Fireable::fire` body is the spec. Add unit tests in `config/tests.rs` or `systems/tests.rs` that exercise the body.

## Common gotchas

- **Forgetting `OrderedFloat` on f32 fields.** The compiler won't complain immediately, but `Hash`/`Eq` derives will fail and you'll get a confusing error somewhere downstream when `EffectType` tries to derive `Hash`.
- **Forgetting the `reverse_all_by_source` override for stack passives.** The default just calls `reverse` once, so a chip that stacked the effect three times would only reverse one instance on disarm. Override the method whenever the effect uses `EffectStack<C>`.
- **Forgetting to call `EffectV3Plugin::build`'s register list.** If you add a new effect's tick systems via `register` but forget to add `MyEffectConfig::register(app)` to the plugin, the systems are silently dropped.
- **Wiring effects into bridges instead of dispatch.** Bridges translate game messages to `Trigger` dispatches; they are not the place to fire effects directly. If you find yourself adding an effect call to a bridge, you probably want a chip that uses your effect via a `When` trigger instead.
