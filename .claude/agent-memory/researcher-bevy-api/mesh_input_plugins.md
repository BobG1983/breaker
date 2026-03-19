---
name: MeshPlugin and InputPlugin — availability and registration
description: Verified existence, re-export paths, and plugin impl details for MeshPlugin and InputPlugin in Bevy 0.18.1 with "2d" feature
type: reference
---

## MeshPlugin

- **Exists**: yes — `bevy_mesh::MeshPlugin` (re-exported as `bevy::mesh::MeshPlugin`)
- **Source**: `bevy_mesh-0.18.1/src/lib.rs:53`
- **Available under "2d" feature**: yes — `"2d"` → `common_api` → `bevy_mesh`
- **Registered by DefaultPlugins**: yes, conditionally `#[cfg(feature = "bevy_mesh")]`

### What MeshPlugin registers

```rust
impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Mesh>()
            .init_asset::<skinning::SkinnedMeshInverseBindposes>()
            .register_asset_reflect::<Mesh>()
            .add_systems(PostUpdate, mark_3d_meshes_as_changed_if_their_assets_changed.after(AssetEventSystems));
    }
}
```

- Calls `init_asset::<Mesh>()` — NOT `init_resource`
- `init_asset` requires a live `AssetServer` resource (panics without it)
- Also registers `AssetEvent<Mesh>` as a message channel
- Also registers `SkinnedMeshInverseBindposes` asset

### Cannot use `init_resource::<Assets<Mesh>>()` as a substitute

`init_asset` does more than init a resource:
1. Registers the asset with `AssetServer` (handle provider, ID space)
2. Inserts `Assets<A>` as a resource
3. Marks it as ambiguous
4. Adds `AssetEvent<A>` as a message

Plain `init_resource::<Assets<Mesh>>()` skips steps 1, 3, and 4 — it will compile but will be missing AssetServer registration. Use `MeshPlugin` or `AssetPlugin` + `app.init_asset::<Mesh>()` instead.

## InputPlugin

- **Exists**: yes — `bevy_input::InputPlugin` (re-exported as `bevy::input::InputPlugin`)
- **Source**: `bevy_input-0.18.1/src/lib.rs:101`
- **Always included**: `InputPlugin` is unconditional in `DefaultPlugins` (no `#[cfg]` guard)

### What InputPlugin registers (with "2d" feature / keyboard + mouse)

- `KeyboardInput` and `KeyboardFocusLost` messages
- `ButtonInput<KeyCode>` and `ButtonInput<Key>` resources
- `keyboard_input_system` in `PreUpdate` (in `InputSystems` set)
- `MouseButtonInput`, `MouseMotion`, `MouseWheel` messages
- `AccumulatedMouseMotion`, `AccumulatedMouseScroll`, `ButtonInput<MouseButton>` resources
- Mouse input systems in `PreUpdate` (in `InputSystems` set)
- Gamepad/touch inputs conditionally via other features

### Re-export paths (verified)

- `bevy::mesh::MeshPlugin` — via `bevy_internal::mesh` → `bevy_mesh as mesh`
- `bevy::input::InputPlugin` — via `bevy_internal::input` → `bevy_input as input`
- `bevy::prelude::*` does NOT include `MeshPlugin` or `InputPlugin` directly
