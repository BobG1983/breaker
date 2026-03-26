use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Spatial2D};

use super::helpers::*;
use crate::{
    cells::components::{Cell, CellHealth},
    effect::effects::shockwave::system::*,
    shared::{BOLT_LAYER, CELL_LAYER, GameDrawLayer},
};

// =========================================================================
// Part C: shockwave_collision
// =========================================================================

/// Behavior 7: Damages cells within current radius, adds to
/// `ShockwaveAlreadyHit`.
#[test]
fn shockwave_collision_damages_cells_within_radius() {
    let mut app = test_app();
    app.add_systems(
        FixedUpdate,
        shockwave_collision.after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
    );

    // Spawn a cell at (30, 0)
    let cell = spawn_cell(&mut app, 30.0, 0.0, 20.0);

    // Let quadtree update
    tick(&mut app);

    // Spawn shockwave at origin with radius already covering the cell
    let sw_bolt = spawn_bolt(&mut app, 0.0, 0.0);
    app.world_mut().spawn((
        Position2D(Vec2::new(0.0, 0.0)),
        ShockwaveRadius {
            current: 50.0,
            max: 96.0,
        },
        ShockwaveDamage {
            damage: 10.0,
            source_chip: None,
            source_bolt: Some(sw_bolt),
        },
        ShockwaveAlreadyHit::default(),
    ));

    tick(&mut app);

    let captured = app.world().resource::<CapturedDamage>();
    assert_eq!(
        captured.0.len(),
        1,
        "cell at distance 30 should be hit by shockwave with radius 50, got {} hits",
        captured.0.len()
    );
    assert_eq!(captured.0[0].cell, cell);
    assert!(
        (captured.0[0].damage - 10.0).abs() < f32::EPSILON,
        "damage should be 10.0, got {}",
        captured.0[0].damage
    );
}

/// Behavior 8: Skips already-hit cells.
#[test]
fn shockwave_collision_skips_already_hit_cells() {
    let mut app = test_app();
    app.add_systems(
        FixedUpdate,
        shockwave_collision.after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
    );

    let cell = spawn_cell(&mut app, 30.0, 0.0, 20.0);

    // Let quadtree update
    tick(&mut app);

    let sw_bolt = spawn_bolt(&mut app, 0.0, 0.0);
    let mut already_hit = HashSet::new();
    already_hit.insert(cell);
    app.world_mut().spawn((
        Position2D(Vec2::new(0.0, 0.0)),
        ShockwaveRadius {
            current: 50.0,
            max: 96.0,
        },
        ShockwaveDamage {
            damage: 10.0,
            source_chip: None,
            source_bolt: Some(sw_bolt),
        },
        ShockwaveAlreadyHit(already_hit),
    ));

    tick(&mut app);

    let captured = app.world().resource::<CapturedDamage>();
    assert_eq!(
        captured.0.len(),
        0,
        "already-hit cell should not receive DamageCell again, got {} hits",
        captured.0.len()
    );
}

/// Behavior 9: Skips locked cells.
#[test]
fn shockwave_collision_skips_locked_cells() {
    let mut app = test_app();
    app.add_systems(
        FixedUpdate,
        shockwave_collision.after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
    );

    let _locked = spawn_locked_cell(&mut app, 30.0, 0.0, 20.0);

    // Let quadtree update
    tick(&mut app);

    let sw_bolt = spawn_bolt(&mut app, 0.0, 0.0);
    app.world_mut().spawn((
        Position2D(Vec2::new(0.0, 0.0)),
        ShockwaveRadius {
            current: 50.0,
            max: 96.0,
        },
        ShockwaveDamage {
            damage: 10.0,
            source_chip: None,
            source_bolt: Some(sw_bolt),
        },
        ShockwaveAlreadyHit::default(),
    ));

    tick(&mut app);

    let captured = app.world().resource::<CapturedDamage>();
    assert_eq!(
        captured.0.len(),
        0,
        "locked cell should not receive DamageCell, got {} hits",
        captured.0.len()
    );
}

/// Behavior 10: Cell without `Aabb2D` is invisible to quadtree query.
#[test]
fn shockwave_collision_only_finds_cells_via_quadtree() {
    let mut app = test_app();
    app.add_systems(
        FixedUpdate,
        shockwave_collision.after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
    );

    // Bare cell without Aabb2D — not in quadtree
    app.world_mut().spawn((
        Cell,
        CellHealth::new(20.0),
        Position2D(Vec2::new(10.0, 0.0)),
        Spatial2D,
        GameDrawLayer::Cell,
    ));

    // Properly registered cell
    let registered = spawn_cell(&mut app, 20.0, 0.0, 20.0);

    // Let quadtree update
    tick(&mut app);

    let sw_bolt = spawn_bolt(&mut app, 0.0, 0.0);
    app.world_mut().spawn((
        Position2D(Vec2::new(0.0, 0.0)),
        ShockwaveRadius {
            current: 50.0,
            max: 96.0,
        },
        ShockwaveDamage {
            damage: 10.0,
            source_chip: None,
            source_bolt: Some(sw_bolt),
        },
        ShockwaveAlreadyHit::default(),
    ));

    tick(&mut app);

    let captured = app.world().resource::<CapturedDamage>();
    assert_eq!(
        captured.0.len(),
        1,
        "only the registered cell (in quadtree) should be hit — bare cell invisible; got {}",
        captured.0.len()
    );
    assert_eq!(
        captured.0[0].cell, registered,
        "DamageCell should target the registered cell"
    );
}

/// Behavior 11: Shockwave uses `CollisionLayers::new(0, CELL_LAYER)`,
/// so bolts are not hit.
#[test]
fn shockwave_collision_does_not_hit_bolt_entities() {
    let mut app = test_app();
    app.add_systems(
        FixedUpdate,
        shockwave_collision.after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
    );

    // Spawn a bolt entity with AABB (as if it were in the quadtree on BOLT_LAYER)
    app.world_mut().spawn((
        Position2D(Vec2::new(10.0, 0.0)),
        Aabb2D::new(Vec2::ZERO, Vec2::new(8.0, 8.0)),
        CollisionLayers::new(BOLT_LAYER, CELL_LAYER),
        Spatial2D,
    ));

    // Also spawn a proper cell to confirm collision works
    let cell = spawn_cell(&mut app, 20.0, 0.0, 20.0);

    // Let quadtree update
    tick(&mut app);

    let sw_bolt = spawn_bolt(&mut app, 0.0, 0.0);
    app.world_mut().spawn((
        Position2D(Vec2::new(0.0, 0.0)),
        ShockwaveRadius {
            current: 50.0,
            max: 96.0,
        },
        ShockwaveDamage {
            damage: 10.0,
            source_chip: None,
            source_bolt: Some(sw_bolt),
        },
        ShockwaveAlreadyHit::default(),
    ));

    tick(&mut app);

    let captured = app.world().resource::<CapturedDamage>();
    assert_eq!(
        captured.0.len(),
        1,
        "only the cell should be hit, not the bolt entity; got {} hits",
        captured.0.len()
    );
    assert_eq!(
        captured.0[0].cell, cell,
        "DamageCell should target the cell, not the bolt"
    );
}

// =========================================================================
// Part E: Integration
// =========================================================================

/// Behavior 13: Multi-tick wavefront — inner cells hit before outer cells.
///
/// Spawns cells at distance 20 and 80. After a few ticks the wavefront
/// should have reached the inner cell but not yet the outer one.
#[test]
fn multi_tick_wavefront_hits_inner_cells_before_outer() {
    let mut app = test_app();
    app.add_systems(
        FixedUpdate,
        (
            tick_shockwave,
            shockwave_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree)
                .after(tick_shockwave),
        ),
    );

    let inner_cell = spawn_cell(&mut app, 20.0, 0.0, 20.0);
    let outer_cell = spawn_cell(&mut app, 80.0, 0.0, 20.0);

    // Let quadtree update
    tick(&mut app);

    // Spawn shockwave: speed=400, max=96
    // After 1 tick: radius = 400/64 ~= 6.25 (inner cell NOT hit yet)
    // After 4 ticks: radius ~= 25.0 (inner cell at 20 IS hit, outer at 80 NOT)
    let sw_bolt = spawn_bolt(&mut app, 0.0, 0.0);
    app.world_mut().spawn((
        Position2D(Vec2::new(0.0, 0.0)),
        ShockwaveRadius {
            current: 0.0,
            max: 96.0,
        },
        ShockwaveSpeed(400.0),
        ShockwaveDamage {
            damage: 10.0,
            source_chip: None,
            source_bolt: Some(sw_bolt),
        },
        ShockwaveAlreadyHit::default(),
    ));

    // Tick 4 times: radius ~= 4 * 6.25 = 25.0
    for _ in 0..4 {
        tick(&mut app);
    }

    let captured = app.world().resource::<CapturedDamage>();

    // Inner cell at distance 20 should be hit (radius ~25)
    let hit_inner = captured.0.iter().any(|m| m.cell == inner_cell);
    assert!(
        hit_inner,
        "inner cell at distance 20 should be hit after radius expands to ~25"
    );

    // Outer cell at distance 80 should NOT be hit yet
    let hit_outer = captured.0.iter().any(|m| m.cell == outer_cell);
    assert!(
        !hit_outer,
        "outer cell at distance 80 should NOT be hit yet (radius ~25)"
    );
}

/// Behavior 14: Dangling `source_bolt` is acceptable — bolt may be despawned
/// while the shockwave entity is still alive.
#[test]
fn dangling_source_bolt_does_not_panic() {
    let mut app = test_app();
    app.add_systems(
        FixedUpdate,
        shockwave_collision.after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
    );

    let cell = spawn_cell(&mut app, 30.0, 0.0, 20.0);

    // Let quadtree update
    tick(&mut app);

    // Spawn a bolt, then despawn it — creating a stale entity
    let stale_bolt = app.world_mut().spawn_empty().id();
    app.world_mut().despawn(stale_bolt);

    app.world_mut().spawn((
        Position2D(Vec2::new(0.0, 0.0)),
        ShockwaveRadius {
            current: 50.0,
            max: 96.0,
        },
        ShockwaveDamage {
            damage: 10.0,
            source_chip: None,
            source_bolt: Some(stale_bolt),
        },
        ShockwaveAlreadyHit::default(),
    ));

    // Should not panic even though source_bolt is a stale entity
    tick(&mut app);

    let captured = app.world().resource::<CapturedDamage>();
    assert_eq!(
        captured.0.len(),
        1,
        "cell should still be damaged even with a stale source_bolt; got {} hits",
        captured.0.len()
    );
    assert_eq!(captured.0[0].cell, cell);
    assert_eq!(
        captured.0[0].source_bolt,
        Some(stale_bolt),
        "DamageCell.source_bolt should carry the stale entity (no panic)"
    );
}
