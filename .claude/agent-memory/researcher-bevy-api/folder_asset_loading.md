---
name: Folder Asset Loading — Bevy 0.18.1
description: LoadedFolder, load_folder(), UntypedHandle.typed(), is_loaded_with_dependencies(), RecursiveDependencyLoadState, iyes_progress FreelyMutableState bound
type: reference
---

## Verified against: Bevy 0.18.1 (docs.rs + bevy GitHub v0.18.1 + iyes_progress v0.16.0 GitHub)

---

## LoadedFolder

```rust
// bevy::asset::LoadedFolder
pub struct LoadedFolder {
    pub handles: Vec<UntypedHandle>,
}
```

- Module: `bevy::asset::LoadedFolder`
- Import: `use bevy::asset::LoadedFolder;` or `use bevy::prelude::*;` (it's in the prelude)
- Implements `Asset` (so it IS an asset that can be tracked via `Assets<LoadedFolder>`)
- `handles` contains one `UntypedHandle` per file found in the folder (recursively)

---

## AssetServer::load_folder()

```rust
pub fn load_folder<'a>(
    &self,
    path: impl Into<AssetPath<'a>>,
) -> Handle<LoadedFolder>
```

- Returns `Handle<LoadedFolder>` (strong handle — keep it alive)
- Loading the same folder twice returns the same handle
- If `file_watcher` feature is enabled, the handle reloads when files change
- Works with standard `AssetPlugin` — no special configuration needed
- Does NOT work with embedded assets (filesystem access required)
- Path is relative to the `assets/` folder

---

## Checking Load Completion

### Option 1: is_loaded_with_dependencies()

```rust
pub fn is_loaded_with_dependencies(
    &self,
    id: impl Into<UntypedAssetId>,
) -> bool
```

Returns `true` only when the folder AND all its dependency assets are fully loaded.
Usage: `asset_server.is_loaded_with_dependencies(folder_handle.id())`

### Option 2: get_recursive_dependency_load_state()

```rust
pub fn get_recursive_dependency_load_state(
    &self,
    id: impl Into<UntypedAssetId>,
) -> Option<RecursiveDependencyLoadState>
```

Returns `None` if the asset isn't tracked yet.

### Option 3: get_load_states() — all three states at once

```rust
pub fn get_load_states(
    &self,
    id: impl Into<UntypedAssetId>,
) -> Option<(LoadState, DependencyLoadState, RecursiveDependencyLoadState)>
```

---

## RecursiveDependencyLoadState

```rust
// bevy::asset::RecursiveDependencyLoadState
pub enum RecursiveDependencyLoadState {
    NotLoaded,
    Loading,
    Loaded,      // entire dependency tree is loaded
    Failed(Arc<AssetLoadError>),
}
```

Methods: `.is_loading() -> bool`, `.is_loaded() -> bool`, `.is_failed() -> bool`

The `Loaded` variant means ALL recursive dependencies (i.e., all files in the folder) are done.

---

## UntypedHandle → Handle<T> conversion

```rust
// All three take `self` (consuming)
pub fn typed<A: Asset>(self) -> Handle<A>          // panics on TypeId mismatch
pub fn try_typed<A: Asset>(self) -> Result<Handle<A>, UntypedAssetConversionError>  // safe
pub fn typed_debug_checked<A: Asset>(self) -> Handle<A>  // panics only in debug builds
pub fn typed_unchecked<A: Asset>(self) -> Handle<A>      // no check at all
```

- Import: `use bevy::asset::UntypedHandle;`
- `typed()` CONSUMES the `UntypedHandle` (takes `self`)
- To iterate and keep the original Vec, clone handles first or collect into a new Vec
- `try_typed()` is the safe option when the type isn't guaranteed

---

## Handle<T>::id()

```rust
pub fn id(&self) -> AssetId<A>
```

Takes `&self`, returns `AssetId<A>`. Used to pass into `is_loaded_with_dependencies()`.

---

## iyes_progress: FreelyMutableState bound

The `track_progress::<S>()` method is on the `ProgressReturningSystem` trait:

```rust
pub trait ProgressReturningSystem<T, Params> {
    fn track_progress<S: FreelyMutableState>(self) -> SystemConfigs;
    fn track_progress_and_stop<S: FreelyMutableState>(self) -> SystemConfigs;
}
```

`ProgressPlugin<S>` also requires `S: FreelyMutableState`:
```rust
pub struct ProgressPlugin<S: FreelyMutableState> { ... }
```

`FreelyMutableState` definition:
```rust
pub trait FreelyMutableState: States { ... }
```

- It's a SUBTRAIT of `States`
- It is automatically implemented as part of `#[derive(States)]` for ordinary state enums
- `ComputedStates` and `SubStates` do NOT implement `FreelyMutableState`
- Any state derived with `#[derive(States)]` satisfies `FreelyMutableState`
- Import: `use bevy::prelude::FreelyMutableState;` (it's in the prelude)

---

## Complete Import Set

```rust
use bevy::asset::{AssetServer, LoadedFolder, RecursiveDependencyLoadState, UntypedHandle};
use bevy::prelude::*; // includes Handle, Assets, Res, AssetId, FreelyMutableState
```

Or more selectively:
```rust
use bevy::asset::LoadedFolder;
use bevy::asset::UntypedHandle;
use bevy::asset::RecursiveDependencyLoadState;
```

---

## Typical Usage Pattern

```rust
#[derive(Resource)]
struct FolderHandles {
    folder: Handle<LoadedFolder>,
}

fn start_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    let folder = asset_server.load_folder("my_assets");
    commands.insert_resource(FolderHandles { folder });
}

fn check_loaded(
    handles: Res<FolderHandles>,
    asset_server: Res<AssetServer>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    my_assets: Res<Assets<MyAsset>>,
) {
    if asset_server.is_loaded_with_dependencies(handles.folder.id()) {
        let folder = loaded_folders.get(&handles.folder).unwrap();
        for untyped in &folder.handles {
            let handle: Handle<MyAsset> = untyped.clone().typed::<MyAsset>();
            // use handle...
        }
    }
}
```

Note: `typed()` consumes the handle, so `.clone()` first when iterating a shared Vec.
