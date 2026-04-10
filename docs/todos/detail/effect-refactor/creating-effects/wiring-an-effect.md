# Wiring an Effect

Step-by-step checklist for adding a new effect to the system.

## 1. Create the config struct

Create a new file in `src/effect/configs/`. Derive the standard set:

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

## 2. Add the variant to EffectType

In the EffectType enum, add:

```rust
MyEffect(MyEffectConfig),
```

If the effect is reversible, also add it to ReversibleEffectType.

## 3. Implement Fireable

```rust
impl Fireable for MyEffectConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        // your fire logic
    }
}
```

See [effect-api/fireable.md](effect-api/fireable.md) for the contract and constraints.

## 4. Implement Reversible (if reversible)

```rust
impl Reversible for MyEffectConfig {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World) {
        // your reverse logic
    }
}
```

See [effect-api/reversible.md](effect-api/reversible.md) for the contract and constraints.

## 5. If passive: implement PassiveEffect

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

## 6. Add the match arm

In the fire dispatch function, add:

```rust
EffectType::MyEffect(config) => config.fire(entity, source, world),
```

In the reverse dispatch function (if reversible), add:

```rust
ReversibleEffectType::MyEffect(config) => config.reverse(entity, source, world),
```

Every arm is the same shape — `config.fire()` / `config.reverse()`.

## 7. Document

- Add a config description file in `ron-syntax/configs/`
- Add a config type file in `rust-types/configs/`
- Add an effect description file in `ron-syntax/effects/`
- Add a behavioral spec file in `creating-effects/effect-implementations/`
- Update `effects-list.md` and `configs/index.md` in both ron-syntax and rust-types
- If passive: add a concrete stack type file in `rust-types/effect-stacking/`

## Summary

| Step | Files touched |
|------|--------------|
| Config struct | `src/effect/configs/my_effect.rs` (new) |
| EffectType variant | `src/effect/types/effect_type.rs` |
| ReversibleEffectType variant | `src/effect/types/reversible_effect_type.rs` (if reversible) |
| Fireable impl | `src/effect/configs/my_effect.rs` |
| Reversible impl | `src/effect/configs/my_effect.rs` (if reversible) |
| PassiveEffect impl | `src/effect/configs/my_effect.rs` (if passive) |
| Fire dispatch arm | `src/effect/dispatch.rs` |
| Reverse dispatch arm | `src/effect/dispatch.rs` (if reversible) |
| RON syntax docs | `ron-syntax/configs/`, `ron-syntax/effects/` |
| Rust type docs | `rust-types/configs/`, `rust-types/effect-stacking/` (if passive) |
| Behavioral spec | `creating-effects/effect-implementations/` |
