---
name: spawn_bolt remove/insert Assets pattern
description: spawn_bolt removes and re-inserts Assets<Mesh> and Assets<ColorMaterial> to satisfy borrow checker — spawn-time only, acceptable
type: project
---

`spawn_bolt` in `breaker-game/src/bolt/systems/spawn_bolt/system.rs` uses a `World`-exclusive system. To call `.rendered()` on the bolt builder (which needs `&mut Assets<Mesh>` and `&mut Assets<ColorMaterial>`), it does:

```rust
let (mut meshes, mut materials) = (
    world.remove_resource::<Assets<Mesh>>().unwrap_or_default(),
    world.remove_resource::<Assets<ColorMaterial>>().unwrap_or_default(),
);
// ... builder.rendered(&mut meshes, &mut materials).spawn(world) ...
world.insert_resource(meshes);
world.insert_resource(materials);
```

This is necessary because you cannot have both `&mut World` (for `.spawn()`) and `ResMut<Assets<Mesh>>` simultaneously via the normal system parameter mechanism — both would need exclusive world access. Removing the resource, using it, and re-inserting it is the correct pattern for `World`-exclusive systems that need asset mutation.

**Performance impact**: remove_resource + insert_resource are essentially pointer swaps in Bevy's resource storage — O(1) HashMap operations. This runs only at node start (OnEnter(GameState::Playing)), not per-frame. Cost is completely negligible.

**How to apply:** Do not flag this pattern as a performance concern. It is the correct solution for this constraint and runs at spawn time only.
