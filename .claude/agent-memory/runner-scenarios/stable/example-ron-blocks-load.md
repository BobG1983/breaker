---
name: Example RON files block folder load
description: .example.ron documentation files placed in bolts/ or breakers/ cause load_folder to fail entirely, hanging all scenarios at frame 0
type: project
---

When `bolt.example.ron` and/or `breaker.example.ron` are present in the `bolts/` and `breakers/` asset directories, `load_folder` encounters them and triggers `AssetLoadError::MissingAssetLoader` (not `MissingAssetLoaderForExtension`). In Bevy 0.18.1, only `MissingAssetLoaderForExtension` and `MissingAssetLoaderForTypeName` are silently skipped in `load_folder_internal` — `MissingAssetLoader` causes the folder to fail entirely. The `LoadedFolder` is never created, `seed_registry` for `BoltRegistry` and `BreakerRegistry` returns `Progress { done: 0, total: 1 }` forever, the `Loading` state never exits, and every scenario hangs at frame 0 until the 300s wall-clock timeout.

**Why:** The `.example.ron` files have extension `example.ron` which doesn't match any registered loader, triggering the wrong error variant.

**How to apply:** If documentation `.example.ron` files are added to any registry directory (`bolts/`, `breakers/`, `nodes/`, `chips/standard/`, `chips/evolutions/`), every scenario will fail this way. Fix: move example files to a subdirectory that isn't scanned, or register a no-op loader for `example.ron`.

**Key code locations:**
- `bevy_asset-0.18.1/src/server/mod.rs:1079-1083` — the skip logic for MissingAssetLoaderForExtension/TypeName
- `rantzsoft_defaults/src/systems/fns.rs:98` — the load_folder call
- `rantzsoft_defaults/src/systems/fns.rs:124-132` — seed_registry returns Progress { done: 0, total: 1 } when folder never loads
