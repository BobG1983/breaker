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
pub fn compute_globals(mut query: ComputeGlobalsQuery) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Spatial2D;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 13: Root entity GlobalPosition2D = Position2D ──

    #[test]
    fn root_entity_global_position_equals_local() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, compute_globals);

        let entity = app
            .world_mut()
            .spawn((Spatial2D, Position2D(Vec2::new(100.0, 200.0))))
            .id();

        tick(&mut app);

        let global_pos = app.world().get::<GlobalPosition2D>(entity).unwrap();
        assert_eq!(
            global_pos.0,
            Vec2::new(100.0, 200.0),
            "root GlobalPosition2D should equal Position2D"
        );
    }

    // ── Behavior 14: Root entity GlobalRotation2D = Rotation2D ──

    #[test]
    fn root_entity_global_rotation_equals_local() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, compute_globals);

        let entity = app
            .world_mut()
            .spawn((Spatial2D, Rotation2D::from_degrees(45.0)))
            .id();

        tick(&mut app);

        let global_rot = app.world().get::<GlobalRotation2D>(entity).unwrap();
        assert!(
            (global_rot.0.as_radians() - std::f32::consts::FRAC_PI_4).abs() < 1e-5,
            "root GlobalRotation2D should equal Rotation2D (~PI/4), got {}",
            global_rot.0.as_radians()
        );
    }

    // ── Behavior 15: Root entity GlobalScale2D = Scale2D ──

    #[test]
    fn root_entity_global_scale_equals_local() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, compute_globals);

        let entity = app
            .world_mut()
            .spawn((Spatial2D, Scale2D { x: 2.0, y: 3.0 }))
            .id();

        tick(&mut app);

        let global_scale = app.world().get::<GlobalScale2D>(entity).unwrap();
        assert!(
            (global_scale.x - 2.0).abs() < f32::EPSILON,
            "root GlobalScale2D.x should be 2.0, got {}",
            global_scale.x
        );
        assert!(
            (global_scale.y - 3.0).abs() < f32::EPSILON,
            "root GlobalScale2D.y should be 3.0, got {}",
            global_scale.y
        );
    }

    // ── Behavior 16: Relative child GlobalPosition2D = parent + child ──

    #[test]
    fn relative_child_global_position_adds_parent() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, compute_globals);

        let parent = app
            .world_mut()
            .spawn((Spatial2D, Position2D(Vec2::new(100.0, 0.0))))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Spatial2D,
                Position2D(Vec2::new(50.0, 0.0)),
                PositionPropagation::Relative,
            ))
            .id();

        tick(&mut app);

        let global_pos = app.world().get::<GlobalPosition2D>(child).unwrap();
        assert_eq!(
            global_pos.0,
            Vec2::new(150.0, 0.0),
            "relative child GlobalPosition2D should be parent(100) + child(50) = 150"
        );
    }

    // ── Behavior 17: Relative child GlobalRotation2D = parent * child ──

    #[test]
    fn relative_child_global_rotation_combines_parent() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, compute_globals);

        let parent = app
            .world_mut()
            .spawn((Spatial2D, Rotation2D::from_degrees(90.0)))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Spatial2D,
                Rotation2D::from_degrees(45.0),
                RotationPropagation::Relative,
            ))
            .id();

        tick(&mut app);

        let global_rot = app.world().get::<GlobalRotation2D>(child).unwrap();
        let expected_radians = 135.0_f32.to_radians();
        assert!(
            (global_rot.0.as_radians() - expected_radians).abs() < 1e-4,
            "relative child GlobalRotation2D should be ~135 degrees, got {} radians",
            global_rot.0.as_radians()
        );
    }

    // ── Behavior 18: Relative child GlobalScale2D = parent * child ──

    #[test]
    fn relative_child_global_scale_multiplies_parent() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, compute_globals);

        let parent = app
            .world_mut()
            .spawn((Spatial2D, Scale2D { x: 2.0, y: 2.0 }))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Spatial2D,
                Scale2D { x: 3.0, y: 4.0 },
                ScalePropagation::Relative,
            ))
            .id();

        tick(&mut app);

        let global_scale = app.world().get::<GlobalScale2D>(child).unwrap();
        assert!(
            (global_scale.x - 6.0).abs() < f32::EPSILON,
            "relative child GlobalScale2D.x should be 2*3=6, got {}",
            global_scale.x
        );
        assert!(
            (global_scale.y - 8.0).abs() < f32::EPSILON,
            "relative child GlobalScale2D.y should be 2*4=8, got {}",
            global_scale.y
        );
    }

    // ── Regression: Grandchild global position accumulates full hierarchy ──

    #[test]
    fn grandchild_global_position_accumulates_full_hierarchy() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, compute_globals);

        let root = app
            .world_mut()
            .spawn((Spatial2D, Position2D(Vec2::new(100.0, 0.0))))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(root),
                Spatial2D,
                Position2D(Vec2::new(50.0, 0.0)),
                PositionPropagation::Relative,
            ))
            .id();

        let grandchild = app
            .world_mut()
            .spawn((
                ChildOf(child),
                Spatial2D,
                Position2D(Vec2::new(10.0, 0.0)),
                PositionPropagation::Relative,
            ))
            .id();

        tick(&mut app);

        let global_pos = app.world().get::<GlobalPosition2D>(grandchild).unwrap();
        // Correct: root(100) + child(50) + grandchild(10) = 160
        // Bug: parent_cache only contains root entities, so grandchild's parent
        // (the child) is not in the cache, falling back to local position (10, 0).
        assert_eq!(
            global_pos.0,
            Vec2::new(160.0, 0.0),
            "grandchild GlobalPosition2D should be root(100) + child(50) + grandchild(10) = 160, \
             got {:?} — parent_cache likely only contains root entities",
            global_pos.0
        );
    }

    // ── Behavior 19: Absolute child GlobalPosition2D = child local ──

    #[test]
    fn absolute_child_global_position_ignores_parent() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, compute_globals);

        let parent = app
            .world_mut()
            .spawn((Spatial2D, Position2D(Vec2::new(100.0, 0.0))))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Spatial2D,
                Position2D(Vec2::new(50.0, 0.0)),
                PositionPropagation::Absolute,
            ))
            .id();

        tick(&mut app);

        let global_pos = app.world().get::<GlobalPosition2D>(child).unwrap();
        assert_eq!(
            global_pos.0,
            Vec2::new(50.0, 0.0),
            "absolute child GlobalPosition2D should equal child local (50, 0), ignoring parent"
        );
    }

    // ── Behavior 20: Absolute child GlobalRotation2D = child local ──

    #[test]
    fn absolute_child_global_rotation_ignores_parent() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, compute_globals);

        let parent = app
            .world_mut()
            .spawn((Spatial2D, Rotation2D::from_degrees(90.0)))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Spatial2D,
                Rotation2D::from_degrees(45.0),
                RotationPropagation::Absolute,
            ))
            .id();

        tick(&mut app);

        let global_rot = app.world().get::<GlobalRotation2D>(child).unwrap();
        let expected_radians = 45.0_f32.to_radians();
        assert!(
            (global_rot.0.as_radians() - expected_radians).abs() < 1e-4,
            "absolute child GlobalRotation2D should be ~45 degrees, got {} radians",
            global_rot.0.as_radians()
        );
    }

    // ── Behavior 21: Absolute child GlobalScale2D = child local ──

    #[test]
    fn absolute_child_global_scale_ignores_parent() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, compute_globals);

        let parent = app
            .world_mut()
            .spawn((Spatial2D, Scale2D { x: 2.0, y: 2.0 }))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Spatial2D,
                Scale2D { x: 3.0, y: 4.0 },
                ScalePropagation::Absolute,
            ))
            .id();

        tick(&mut app);

        let global_scale = app.world().get::<GlobalScale2D>(child).unwrap();
        assert!(
            (global_scale.x - 3.0).abs() < f32::EPSILON,
            "absolute child GlobalScale2D.x should be 3.0, got {}",
            global_scale.x
        );
        assert!(
            (global_scale.y - 4.0).abs() < f32::EPSILON,
            "absolute child GlobalScale2D.y should be 4.0, got {}",
            global_scale.y
        );
    }
}
