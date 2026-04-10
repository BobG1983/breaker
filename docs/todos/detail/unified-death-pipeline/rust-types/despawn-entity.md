# Name
DespawnEntity

# Syntax
```rust
#[derive(Message, Clone, Debug)]
struct DespawnEntity {
    entity: Entity,
}
```

# Description
Deferred despawn request. Sent after death animations and trigger evaluation complete. Processed by `process_despawn_requests` in PostFixedUpdate.

Lives in shared/messages.rs — not domain-specific.

DO send DespawnEntity instead of calling `commands.entity(e).despawn()` directly in death handling. The entity must survive through the full death chain.
DO NOT send DespawnEntity for entities that aren't dying — this is a death pipeline message, not a general-purpose despawn.
