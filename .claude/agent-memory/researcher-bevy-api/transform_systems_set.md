---
name: TransformSystems system set
description: TransformSystems enum in Bevy 0.18.1 — name, variants, import path. NOT TransformSystem (old name).
type: reference
---

## TransformSystems (verified bevy_transform 0.18.1 source: plugins.rs)

**Bevy 0.18.1 uses `TransformSystems` (plural), NOT `TransformSystem` (singular).**

```rust
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum TransformSystems {
    /// Propagates changes in transform to children's GlobalTransform
    Propagate,
}
```

### Import path

```rust
use bevy::transform::TransformSystems;
// OR
use bevy::transform::plugins::TransformSystems;
```

`TransformSystems` is also re-exported in `bevy::prelude`.

### The one variant

- `TransformSystems::Propagate` — the single variant; tags `sync_simple_transforms`, `propagate_parent_transforms`, and `mark_dirty_trees` systems

These systems run in both `PostStartup` and `PostUpdate` schedules.

### Common ordering pattern

```rust
// Run my system before transform propagation
app.add_systems(PostUpdate,
    my_system.before(TransformSystems::Propagate)
);

// Run my system after transform propagation
app.add_systems(PostUpdate,
    my_system.after(TransformSystems::Propagate)
);
```

### What does NOT exist

- `TransformSystem::TransformPropagate` — this was the 0.13/0.14-era name
- `TransformSystem` (singular) — does not exist in 0.18.1
- `TransformSystems::TransformPropagate` — variant is just `Propagate`, not `TransformPropagate`
