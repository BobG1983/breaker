---
name: Bevy 0.18 Hierarchy API
description: ChildOf, ChildSpawner, with_children — parent/child entity hierarchy patterns verified against v0.18.1 source
type: reference
---

## Hierarchy API (verified v0.18.1 source: bevy_ecs-0.18.1/src/hierarchy.rs)

- The parent component is `ChildOf`, NOT `Parent` — `Parent` does not exist in 0.18.1
- `ChildOf` is in `bevy::prelude`; `bevy_hierarchy` crate no longer exists (merged into `bevy_ecs`)
- Definition: `pub struct ChildOf(#[entities] pub Entity);` — tuple struct wrapping the parent `Entity`
- `#[doc(alias = "Parent")]` on `ChildOf` — confirms `Parent` was renamed to `ChildOf`
- Method: `pub fn parent(&self) -> Entity` — returns the parent Entity
- Direct field access also works: `child_of.0`
- In queries: `Query<&ChildOf, With<MyMarker>>`; call `.parent()` on the result
- `Children` component: lives on the PARENT, contains `Vec<Entity>` of child entity ids
- Hierarchy is maintained automatically via component hooks — never manually mutate `Children`
- `ChildOf` self-removes if parent is despawned or if entity tries to parent itself (hooks validate)
- Spawn pattern: `world.spawn(ChildOf(parent_entity))` or via `commands.entity(parent).with_child(bundle)`
- `with_child(bundle)` on `EntityCommands` spawns one child and inserts `ChildOf` automatically
- `children![]` macro: `world.spawn((Name::new("Root"), children![Name::new("Child1")]))`

## with_children closure parameter (verified v0.18.0 docs.rs)

- `EntityCommands::with_children` signature:
  `pub fn with_children(&mut self, func: impl FnOnce(&mut RelatedSpawnerCommands<'_, ChildOf>)) -> &mut EntityCommands<'a>`
- `EntityWorldMut::with_children` signature:
  `pub fn with_children(&mut self, func: impl FnOnce(&mut RelatedSpawner<'_, ChildOf>)) -> &mut EntityWorldMut<'w>`
- `ChildSpawner<'w>` is a type alias: `pub type ChildSpawner<'w> = RelatedSpawner<'w, ChildOf>;`
- `ChildSpawnerCommands<'_>` is a type alias: `pub type ChildSpawnerCommands<'_> = RelatedSpawnerCommands<'_, ChildOf>;`
- Use `ChildSpawner` in function signatures taking the `EntityWorldMut::with_children` parent
- Use `ChildSpawnerCommands` in function signatures taking the `EntityCommands::with_children` parent
- Import: `bevy::ecs::hierarchy::{ChildSpawner, ChildSpawnerCommands}` (NOT in prelude)
- `ChildBuilder` does NOT exist in 0.18 — that name is from Bevy 0.14 and earlier
- Example function signature: `fn spawn_children(parent: &mut ChildSpawnerCommands<'_>) { parent.spawn(...); }`
