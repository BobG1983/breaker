//! Watcher system that stamps `SpawnStampRegistry` entries onto newly spawned
//! cell entities.

use bevy::prelude::*;

use super::super::resource::SpawnStampRegistry;
use crate::{
    cells::components::Cell,
    effect_v3::{commands::EffectCommandsExt, types::EntityKind},
};

/// Stamps registered `(name, tree)` pairs into `BoundEffects` on every newly
/// spawned `Cell` entity.
///
/// Iterates the `SpawnStampRegistry.entries` for each newly-added `Cell` and
/// delegates to `commands.stamp_effect` for every entry whose `EntityKind`
/// exactly matches `EntityKind::Cell`. `EntityKind::Any` entries are ignored —
/// wildcarding is reserved for trigger-side matching, not spawn-time stamping.
pub(crate) fn stamp_spawned_cells(
    registry: Res<SpawnStampRegistry>,
    new_cells: Query<Entity, Added<Cell>>,
    mut commands: Commands,
) {
    const KIND: EntityKind = EntityKind::Cell;

    if registry.entries.is_empty() {
        return;
    }

    for entity in &new_cells {
        for (entry_kind, name, tree) in &registry.entries {
            if *entry_kind == KIND {
                commands.stamp_effect(entity, name.clone(), tree.clone());
            }
        }
    }
}
