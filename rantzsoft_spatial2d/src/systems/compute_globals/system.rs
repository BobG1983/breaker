//! Computes `GlobalPosition2D`, `GlobalRotation2D`, `GlobalScale2D` from
//! local values and parent hierarchy.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    components::{
        GlobalPosition2D, GlobalRotation2D, GlobalScale2D, Position2D, Rotation2D, Scale2D,
    },
    propagation::{PositionPropagation, RotationPropagation, ScalePropagation},
};

/// Query type for `compute_globals` — avoids clippy `type_complexity`.
type ComputeGlobalsQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position2D,
        &'static Rotation2D,
        &'static Scale2D,
        &'static mut GlobalPosition2D,
        &'static mut GlobalRotation2D,
        &'static mut GlobalScale2D,
        Option<&'static ChildOf>,
        Option<&'static PositionPropagation>,
        Option<&'static RotationPropagation>,
        Option<&'static ScalePropagation>,
    ),
>;

/// Computes global position, rotation, and scale from local values and parent
/// hierarchy. Root entities copy local to global. Children combine with parent
/// globals according to their propagation mode (`Relative` or `Absolute`).
pub(crate) fn compute_globals(mut query: ComputeGlobalsQuery) {
    // Collect globals in a temporary map so children can read parent values
    // without conflicting mutable borrows.
    let mut parent_cache: HashMap<Entity, (Vec2, Rot2, (f32, f32))> = HashMap::new();

    // Pass 1: roots (no `ChildOf`) copy local to global and cache values.
    for (entity, pos, rot, scale, mut g_pos, mut g_rot, mut g_scale, child_of, ..) in &mut query {
        if child_of.is_some() {
            continue;
        }
        g_pos.0 = pos.0;
        g_rot.0 = rot.0;
        g_scale.x = scale.x;
        g_scale.y = scale.y;
        parent_cache.insert(entity, (pos.0, rot.0, (scale.x, scale.y)));
    }

    // Pass 2+: iterate children whose parent is in cache, compute globals,
    // insert into cache. Repeat until no new entries are added (handles
    // multi-level hierarchies: grandchildren, great-grandchildren, etc.).
    let mut made_progress = true;
    while made_progress {
        made_progress = false;
        for (
            entity,
            pos,
            rot,
            scale,
            mut g_pos,
            mut g_rot,
            mut g_scale,
            child_of,
            pos_prop,
            rot_prop,
            scale_prop,
        ) in &mut query
        {
            let Some(child_of) = child_of else {
                continue;
            };
            // Skip already-processed entities.
            if parent_cache.contains_key(&entity) {
                continue;
            }
            let Some(&(parent_pos, parent_rot, (parent_scale_x, parent_scale_y))) =
                parent_cache.get(&child_of.parent())
            else {
                continue;
            };

            // Position
            let my_pos = if pos_prop.is_some_and(|p| *p == PositionPropagation::Absolute) {
                pos.0
            } else {
                parent_pos + pos.0
            };
            g_pos.0 = my_pos;

            // Rotation
            let my_rot = if rot_prop.is_some_and(|p| *p == RotationPropagation::Absolute) {
                rot.0
            } else {
                parent_rot * rot.0
            };
            g_rot.0 = my_rot;

            // Scale
            let (my_scale_x, my_scale_y) =
                if scale_prop.is_some_and(|p| *p == ScalePropagation::Absolute) {
                    (scale.x, scale.y)
                } else {
                    (parent_scale_x * scale.x, parent_scale_y * scale.y)
                };
            g_scale.x = my_scale_x;
            g_scale.y = my_scale_y;

            parent_cache.insert(entity, (my_pos, my_rot, (my_scale_x, my_scale_y)));
            made_progress = true;
        }
    }

    // Final fallback: any children whose parent was never found in the cache
    // (orphaned hierarchy) fall back to their local values.
    for (entity, pos, rot, scale, mut g_pos, mut g_rot, mut g_scale, child_of, ..) in &mut query {
        if child_of.is_none() {
            continue;
        }
        if parent_cache.contains_key(&entity) {
            continue;
        }
        g_pos.0 = pos.0;
        g_rot.0 = rot.0;
        g_scale.x = scale.x;
        g_scale.y = scale.y;
    }
}
