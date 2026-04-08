//! System to slide guardian cells around their parent's ring.

use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::cells::behaviors::guarded::components::{
    GuardedCell, GuardianCell, GuardianGridStep, GuardianSlideSpeed, GuardianSlot, SlideTarget,
    ring_slot_offset,
};

/// Slides guardian cells toward their target ring slot each fixed timestep.
///
/// Uses a two-pass algorithm:
/// 1. First pass (immutable): collect a map of parent to list of (guardian, `current_slot`).
/// 2. Second pass (mutable): for each guardian, compute target world position and move toward it.
///    When within snap distance, snap to exact position, update slot, and pick the next
///    clockwise unoccupied target slot.
type GuardianQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Position2D,
        &'static mut GuardianSlot,
        &'static mut SlideTarget,
        &'static GuardianSlideSpeed,
        &'static GuardianGridStep,
        &'static ChildOf,
    ),
    (With<GuardianCell>, Without<GuardedCell>),
>;

pub(crate) fn slide_guardian_cells(
    time: Res<Time<Fixed>>,
    guarded_query: Query<&Position2D, With<GuardedCell>>,
    mut guardian_query: GuardianQuery,
) {
    let dt = time.delta_secs();

    // First pass: collect parent -> guardians mapping (immutable iteration).
    let mut parent_to_guardians: HashMap<Entity, Vec<(Entity, u8)>> = HashMap::new();
    for (entity, _, slot, _, _, _, child_of) in guardian_query.iter() {
        parent_to_guardians
            .entry(child_of.parent())
            .or_default()
            .push((entity, slot.0));
    }

    // Second pass: move each guardian toward its target.
    for (entity, mut pos, mut slot, mut slide_target, speed, grid_step, child_of) in
        &mut guardian_query
    {
        if speed.0 <= 0.0 {
            continue;
        }

        let Ok(parent_pos) = guarded_query.get(child_of.parent()) else {
            continue;
        };

        let (offset_x, offset_y) = ring_slot_offset(slide_target.0);
        let target_world = Vec2::new(
            offset_x.mul_add(grid_step.step_x, parent_pos.0.x),
            offset_y.mul_add(grid_step.step_y, parent_pos.0.y),
        );

        let diff = target_world - pos.0;
        let distance = diff.length();

        let snap_threshold = 0.5;
        let move_dist = speed.0 * dt;

        if distance <= snap_threshold || move_dist >= distance {
            // Snap to exact target position.
            pos.0 = target_world;
            slot.0 = slide_target.0;

            // Pick next clockwise unoccupied slot.
            let siblings = parent_to_guardians.get(&child_of.parent());
            let mut next_target = (slot.0 + 1) % 8;

            if let Some(siblings) = siblings {
                // Walk clockwise around the ring to find the first unoccupied slot.
                let mut checked = 0u8;
                while checked < 7 {
                    let occupied = siblings
                        .iter()
                        .any(|&(e, s)| e != entity && s == next_target);
                    if !occupied {
                        break;
                    }
                    next_target = (next_target + 1) % 8;
                    checked += 1;
                }
                // If all 7 other slots are occupied, next_target wraps back to current slot.
            }

            slide_target.0 = next_target;
        } else {
            // Move toward target, clamped to remaining distance.
            let direction = diff / distance;
            pos.0 += direction * move_dist;
        }
    }
}
