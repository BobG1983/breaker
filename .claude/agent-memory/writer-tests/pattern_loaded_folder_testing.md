---
name: LoadedFolder testing pattern
description: How to create LoadedFolder with UntypedHandles in tests, Handle::into() for typed-to-untyped conversion
type: project
---

# Testing with LoadedFolder in Bevy 0.18

When testing systems that consume `Assets<LoadedFolder>`, you can manually create a `LoadedFolder` and add it to the assets.

## Setup

```rust
app.init_asset::<bevy::asset::LoadedFolder>();  // May be redundant if AssetPlugin registers it
app.init_asset::<MyAsset>();
```

## Creating typed handles and converting to UntypedHandle

Use `Handle<T>::into()` (not `.untyped()`) to convert a typed handle to `UntypedHandle`:

```rust
let typed_handle: Handle<MyAsset> = assets.add(my_asset);
let untyped: UntypedHandle = typed_handle.into();  // consumes the typed handle
```

`.into()` consumes the handle. If you need both typed and untyped versions, clone first:
```rust
let typed_handle: Handle<MyAsset> = assets.add(my_asset);
let untyped: UntypedHandle = typed_handle.clone().into();
// typed_handle is still usable
```

## Creating a LoadedFolder

```rust
let folder_handle = {
    let mut loaded_folders = app.world_mut().resource_mut::<Assets<bevy::asset::LoadedFolder>>();
    loaded_folders.add(bevy::asset::LoadedFolder {
        handles: vec![h_alpha.into(), h_beta.into()],
    })
};
```

## Why not `.untyped()`

The method name may vary across Bevy versions. `Into<UntypedHandle>` is stable.
