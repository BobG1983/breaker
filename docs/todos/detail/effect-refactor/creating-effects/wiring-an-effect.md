# Wiring an Effect

Step-by-step checklist for adding a new effect to the system.

## 1. Create the effect folder

Create a new folder in `src/effect/effects/<my_effect>/` with at minimum:

```
effects/my_effect/
  mod.rs        # pub(crate) mod config; + re-exports
  config.rs     # config struct + Fireable + Reversible (if reversible) + PassiveEffect (if passive)
```

If the effect has runtime components, add `components.rs`. If it has tick/update systems, add `systems.rs`.

## 2. Create the config struct

In `config.rs`, derive the standard set:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MyEffectConfig {
    power: OrderedFloat<f32>,
}
```

All f32 fields use `OrderedFloat<f32>`. This gives you Eq for EffectStack removal matching.

If the effect has no parameters, use an empty struct:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MyEffectConfig {}
```

## 3. Add the variant to EffectType

In the EffectType enum, add:

```rust
MyEffect(MyEffectConfig),
```

If the effect is reversible, also add it to ReversibleEffectType.

## 4. Implement Fireable

In `config.rs`:

```rust
impl Fireable for MyEffectConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        // your fire logic
    }
}
```

If the effect has runtime systems, override `register`:

```rust
impl Fireable for MyEffectConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        // your fire logic
    }

    fn register(app: &mut App) {
        app.add_systems(FixedUpdate,
            tick_my_effect.in_set(EffectSystems::Tick)
        );
    }
}
```

Effects with no runtime systems (passive effects, fire-and-forget) skip the register override — the default no-op is correct.

See [effect-api/fireable.md](effect-api/fireable.md) for the contract and constraints.

## 5. Implement Reversible (if reversible)

In `config.rs`:

```rust
impl Reversible for MyEffectConfig {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World) {
        // your reverse logic
    }
}
```

See [effect-api/reversible.md](effect-api/reversible.md) for the contract and constraints.

## 6. If passive: implement PassiveEffect

In `config.rs`:

```rust
impl PassiveEffect for MyEffectConfig {
    fn aggregate(entries: &[(String, Self)]) -> f32 {
        entries.iter().map(|(_, config)| config.power.0).product()
    }
}
```

This also requires implementing Fireable and Reversible with the standard push/remove pattern:

- fire: get or insert `EffectStack<MyEffectConfig>`, call `stack.push(source, self.clone())`
- reverse: get `EffectStack<MyEffectConfig>`, call `stack.remove(source, self)`

See [effect-api/passive-effect.md](effect-api/passive-effect.md) for the full pattern.

## 7. Add the match arm

In the fire dispatch function, add:

```rust
EffectType::MyEffect(config) => config.fire(entity, source, world),
```

In the reverse dispatch function (if reversible), add:

```rust
ReversibleEffectType::MyEffect(config) => config.reverse(entity, source, world),
```

Every arm is the same shape — `config.fire()` / `config.reverse()`.

## 8. Document

- Add a config description file in `ron-syntax/configs/`
- Add a config type file in `rust-types/configs/`
- Add an effect description file in `ron-syntax/effects/`
- Add a behavioral spec file in `creating-effects/effect-implementations/`
- Update `effects-list.md` and `configs/index.md` in both ron-syntax and rust-types
- If passive: add a concrete stack type file in `rust-types/effect-stacking/`

## Summary

| Step | Files touched |
|------|--------------|
| Effect folder | `src/effect/effects/my_effect/` (new folder) |
| Config struct | `src/effect/effects/my_effect/config.rs` (new) |
| Fireable impl (fire + register) | `src/effect/effects/my_effect/config.rs` |
| Reversible impl | `src/effect/effects/my_effect/config.rs` (if reversible) |
| PassiveEffect impl | `src/effect/effects/my_effect/config.rs` (if passive) |
| Components | `src/effect/effects/my_effect/components.rs` (if needed) |
| Systems | `src/effect/effects/my_effect/systems.rs` (if needed) |
| Module wiring | `src/effect/effects/my_effect/mod.rs` (new) |
| EffectType variant | `src/effect/types/effect_type.rs` |
| ReversibleEffectType variant | `src/effect/types/reversible_effect_type.rs` (if reversible) |
| Fire dispatch arm | `src/effect/dispatch/fire_dispatch.rs` |
| Reverse dispatch arm | `src/effect/dispatch/reverse_dispatch.rs` (if reversible) |
