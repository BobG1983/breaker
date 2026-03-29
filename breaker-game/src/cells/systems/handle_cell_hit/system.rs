//! System to handle cell damage when hit by the bolt.

use bevy::prelude::*;

use crate::cells::{
    components::Cell,
    messages::{DamageCell, RequestCellDestroyed},
    queries::DamageVisualQuery,
};

/// Handles cell damage in response to [`DamageCell`] messages.
///
/// Decrements cell health, updates visual feedback via material color,
/// and sends [`RequestCellDestroyed`] when cells reach zero HP.
///
/// Guards against the same cell appearing in multiple messages in one frame
/// (e.g., two bolts hitting the same cell simultaneously): only the first hit
/// that destroys the cell is processed; subsequent messages for an already-destroyed
/// cell are skipped to prevent duplicate [`RequestCellDestroyed`] messages.
pub(crate) fn handle_cell_hit(
    mut reader: MessageReader<DamageCell>,
    mut cell_query: Query<DamageVisualQuery, With<Cell>>,
    mut request_destroyed_writer: MessageWriter<RequestCellDestroyed>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut despawned: Local<Vec<Entity>>,
) {
    // Local<Vec> reuses its heap allocation across frames — zero allocs after warmup.
    // Bounded by MAX_BOUNCES hits per frame.
    despawned.clear();
    for msg in reader.read() {
        if despawned.contains(&msg.cell) {
            continue;
        }
        let Ok((
            mut health,
            material_handle,
            visuals,
            is_required,
            is_locked,
            position,
            is_shielded,
        )) = cell_query.get_mut(msg.cell)
        else {
            continue;
        };

        // Locked cells are immune to damage until unlocked.
        if is_locked {
            continue;
        }

        // Shielded cells are immune to damage while shield is active.
        if is_shielded {
            continue;
        }

        let destroyed = health.take_damage(msg.damage);

        if destroyed {
            // Two-phase destruction: write request (entity stays alive for bridge evaluation)
            request_destroyed_writer.write(RequestCellDestroyed {
                cell: msg.cell,
                position: position.0,
                was_required_to_clear: is_required,
            });
            despawned.push(msg.cell);
        } else {
            // Visual feedback — dim HDR intensity based on remaining health
            let frac = health.fraction();
            let intensity = frac * visuals.hdr_base;
            if let Some(material) = materials.get_mut(material_handle.id()) {
                material.color = Color::srgb(
                    intensity,
                    visuals.green_min * frac,
                    visuals.blue_range.mul_add(1.0 - frac, visuals.blue_base),
                );
            }
        }
    }
}
