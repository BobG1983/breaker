# Name
EffectSourceChip

# Struct
```rust
/// Identifies which chip (upgrade) caused this effect entity to be spawned.
/// Used for damage attribution and UI display.
#[derive(Component)]
pub struct EffectSourceChip(pub Option<String>);
```

# Location
`src/effect/components/`

# Description
`EffectSourceChip` is a shared component added to spawned effect entities (shockwaves, chain lightning arcs, gravity wells, etc.) to track which chip was responsible for creating them.

- **Added by**: Effect `fire()` methods attach this to spawned entities, passing the chip name from the trigger context. `None` indicates the effect was not chip-sourced (e.g., from a cell death cascade).
- **Read by**: Damage attribution systems use this to credit kills and flux generation to the originating chip. UI systems use it for combat log display.
- **Removed by**: Removed when the effect entity is despawned (no separate removal logic needed).
