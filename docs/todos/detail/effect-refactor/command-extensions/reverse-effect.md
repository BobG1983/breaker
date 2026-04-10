# Name
reverse_effect

# Struct
```rust
struct ReverseEffectCommand {
    entity: Entity,
    effect: ReversibleEffectType,
    source: String,
}
```

# Description
Undo an effect on an entity.

Look up the entity in the world. Reverse the effect — remove what fire_effect added. For multiplicative effects (SpeedBoost, DamageBoost, SizeBoost, etc.), divide instead of multiply. For spawned entities (Shield, Pulse), despawn them. For boolean toggles (FlashStep), disable them.

The type system enforces that only reversible effects can be passed — ReversibleEffectType is the subset of EffectType that can be cleanly undone.

The source string must match the source used in the original fire_effect call so the correct instance is reversed when multiple sources apply the same effect type.

If the entity does not exist in the world, do nothing.
