---
name: ron_deserialization_patterns
description: Confirmed safe RON deserialization patterns and production panic surface audit
type: project
---

Audited 2026-03-19 (develop, commit 7256360). Updated to reflect ron 0.12 upgrade and new crates.

## Summary

All RON parsing uses the `ron` 0.12 crate via `ron::de::from_str` (typed deserialization with
serde-derived `Deserialize` impls). No custom deserializers. No call site has unvalidated
`from_str` on runtime user input.

## Production path — via Bevy asset system

All game data files are loaded via `RonAssetPlugin` + `bevy_asset_loader` using hard-coded asset
paths (declared in `DefaultsCollection` with `#[asset(path = "...")]` macros). There is no runtime
path construction using user input.

Font paths from RON (`font_path`, `title_font_path`, `menu_font_path`) are passed to
`asset_server.load()`. These paths come from Bevy config RON files, not from user input, and
Bevy's asset server restricts paths to the assets directory. No path traversal risk in practice.

## Production path — scenario runner

The scenario runner uses `fs::read_to_string` + `ron::de::from_str` on scenario files discovered
by walking `scenarios/` at a compile-time-pinned path (`env!("CARGO_MANIFEST_DIR")/scenarios`).
Malformed RON files cause an error log + skip, not a panic (`discovery.rs:73-75` uses `.ok()?`).

## Test-only panics

All `expect()`/`unwrap()` on RON parsing in `cells/resources.rs`, `run/node/definition.rs`,
`breaker/resources.rs`, `bolt/resources.rs`, `chips/definition.rs`, etc. are inside
`#[cfg(test)] mod tests` blocks. They do not execute at runtime.

## Warning: RON hp/regen_rate not validated at runtime

`CellTypeDefinition.hp` and `CellBehavior.regen_rate` are deserialized from `.cell.ron` files
without bounds checks in the asset loader path. A `.cell.ron` with `hp: -1.0` or
`regen_rate: Some(999999.0)` would load silently and cause downstream logic issues. Test code
validates `hp > 0.0` but only in `#[cfg(test)]`.

**Status as of this audit:** Still unvalidated. No runtime validation added yet.

**How to apply:** On future audits, check whether validation has been added to the runtime
asset-loaded path (the system that processes `CellTypeAsset` events and populates `CellTypeRegistry`).
