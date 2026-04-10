# Name
DamageDealt\<T\>

# Syntax
```rust
#[derive(Message, Clone, Debug)]
struct DamageDealt<T: GameEntity> {
    dealer: Option<Entity>,
    target: Entity,
    amount: f32,
    source_chip: Option<String>,
    _marker: PhantomData<T>,
}
```

# Description
Generic damage message — one Bevy message queue per victim type T. Replaces `DamageCell`.

- dealer: The entity that originated this damage. Propagated through effect chains so the final kill is attributed to the original source.
- target: The entity taking the damage.
- amount: Pre-calculated damage amount (includes any multipliers from the sender).
- source_chip: Which chip originated this damage chain, for UI/stats.

Sent by: bolt collision, shockwave fire, chain lightning fire, explode fire, piercing beam fire, tether beam tick, or any effect that deals damage.

`DamageDealt<Cell>` replaces `DamageCell`. `DamageDealt<Wall>`, `DamageDealt<Bolt>` are new.

DO include the dealer from the TriggerContext when firing damage from an effect chain.
DO NOT send DamageDealt for entities that don't have Hp — the apply_damage system will silently skip them.
