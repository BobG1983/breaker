use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Spatial2D};

use super::system::slide_guardian_cells;
use crate::cells::{behaviors::guarded::components::*, components::Cell};

fn test_app() -> App {
    use crate::shared::test_utils::TestAppBuilder;

    TestAppBuilder::new()
        .with_system(FixedUpdate, slide_guardian_cells)
        .build()
}

/// Sets the fixed timestep to `dt` and accumulates one step, then runs update.
fn tick_with_dt(app: &mut App, dt: Duration) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}

/// Spawns a guarded parent cell at the given position.
fn spawn_guarded_parent(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((Cell, GuardedCell, Spatial2D, Position2D(pos)))
        .id()
}

/// Spawns a guardian child entity at the given world position with the given slot/target/speed/step.
fn spawn_guardian(
    app: &mut App,
    parent: Entity,
    world_pos: Vec2,
    slot: u8,
    target: u8,
    speed: f32,
    grid_step: GuardianGridStep,
) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            GuardianCell,
            Spatial2D,
            Position2D(world_pos),
            GuardianSlot(slot),
            SlideTarget(target),
            GuardianSlideSpeed(speed),
            grid_step,
            ChildOf(parent),
        ))
        .id()
}

// ── Section H: slide_guardian_cells System ──────────────────────────────

// Behavior 25: Guardian slides toward target slot at slide_speed
// Slot 3 offset is (1.0, 0.0), slot 4 offset is (1.0, -1.0)
// Parent at origin, step_x=72.0, step_y=26.0
// Guardian at slot 3: Vec2(72.0, 0.0), target slot 4: Vec2(72.0, -26.0)
// Distance = 26.0, speed = 72.0, dt = 0.5s => move = 36.0 > 26.0 => snap
#[test]
fn guardian_slides_toward_target_and_snaps_when_close() {
    let mut app = test_app();
    let parent = spawn_guarded_parent(&mut app, Vec2::ZERO);
    let guardian = spawn_guardian(
        &mut app,
        parent,
        Vec2::new(72.0, 0.0), // slot 3 position
        3,                    // current slot
        4,                    // target slot
        72.0,                 // speed
        GuardianGridStep {
            step_x: 72.0,
            step_y: 26.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_millis(500)); // dt = 0.5s

    // Distance to target = 26.0, move_dist = 72.0 * 0.5 = 36.0 > 26.0
    // Should snap to exact target position Vec2(72.0, -26.0)
    let pos = app.world().get::<Position2D>(guardian).unwrap();
    assert!(
        (pos.0.x - 72.0).abs() < 1.0 && (pos.0.y - (-26.0)).abs() < 1.0,
        "guardian should snap to slot 4 position Vec2(72.0, -26.0), got {:?}",
        pos.0
    );
    let slot = app.world().get::<GuardianSlot>(guardian).unwrap();
    assert_eq!(
        slot.0, 4,
        "GuardianSlot should update to 4 after arrival, got {}",
        slot.0
    );
}

// Behavior 25 edge case + Behavior 51: GuardianSlideSpeed(0.0) means no movement
// Uses a companion moving guardian to ensure the test fails against the no-op stub.
#[test]
fn guardian_with_zero_speed_does_not_move() {
    let mut app = test_app();
    let parent = spawn_guarded_parent(&mut app, Vec2::ZERO);
    // Zero-speed guardian at slot 3
    let stationary = spawn_guardian(
        &mut app,
        parent,
        Vec2::new(72.0, 0.0),
        3,
        4,
        0.0, // zero speed
        GuardianGridStep {
            step_x: 72.0,
            step_y: 26.0,
        },
    );
    // Moving guardian at slot 0 — ensures system must run for test to pass
    let moving = spawn_guardian(
        &mut app,
        parent,
        Vec2::new(-72.0, 26.0),
        0,
        1,
        1000.0, // high speed — should snap to slot 1 position
        GuardianGridStep {
            step_x: 72.0,
            step_y: 26.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs(1));

    // Moving guardian should have moved (proves system ran)
    let moving_pos = app.world().get::<Position2D>(moving).unwrap();
    let moving_slot = app.world().get::<GuardianSlot>(moving).unwrap();
    assert!(
        moving_slot.0 == 1 || (moving_pos.0 - Vec2::new(-72.0, 26.0)).length() > 0.1,
        "moving guardian should have moved or changed slot — proves system actually ran"
    );

    // Stationary guardian should NOT have moved
    let pos = app.world().get::<Position2D>(stationary).unwrap();
    assert!(
        (pos.0.x - 72.0).abs() < f32::EPSILON && (pos.0.y - 0.0).abs() < f32::EPSILON,
        "guardian with zero speed should not move, got {:?}",
        pos.0
    );
    let slot = app.world().get::<GuardianSlot>(stationary).unwrap();
    assert_eq!(
        slot.0, 3,
        "GuardianSlot should remain 3 with zero speed, got {}",
        slot.0
    );
}

// Behavior 26: Guardian arrives at target slot and picks new target
#[test]
fn guardian_arrives_and_picks_new_target() {
    let mut app = test_app();
    let parent = spawn_guarded_parent(&mut app, Vec2::ZERO);
    // Start guardian very close to slot 4 position: Vec2(72.0, -26.0)
    // Current slot is 3, target is 4, speed 100.0
    let guardian = spawn_guardian(
        &mut app,
        parent,
        Vec2::new(72.0, -25.5), // within 0.5 of target
        3,
        4,
        100.0,
        GuardianGridStep {
            step_x: 72.0,
            step_y: 26.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs(1));

    let slot = app.world().get::<GuardianSlot>(guardian).unwrap();
    assert_eq!(
        slot.0, 4,
        "GuardianSlot should update to 4 after arrival, got {}",
        slot.0
    );
    let target = app.world().get::<SlideTarget>(guardian).unwrap();
    // New target should be an adjacent slot (3 or 5)
    assert!(
        target.0 == 3 || target.0 == 5,
        "new SlideTarget should be adjacent to slot 4 (3 or 5), got {}",
        target.0
    );
    let pos = app.world().get::<Position2D>(guardian).unwrap();
    assert!(
        (pos.0.x - 72.0).abs() < f32::EPSILON && (pos.0.y - (-26.0)).abs() < f32::EPSILON,
        "guardian should snap to exact slot 4 position, got {:?}",
        pos.0
    );
}

// Behavior 27: Guardian skips occupied slots when choosing next target
#[test]
fn guardian_skips_occupied_slots_when_choosing_next_target() {
    let mut app = test_app();
    let parent = spawn_guarded_parent(&mut app, Vec2::ZERO);

    // Guardian A at slot 3, sliding toward slot 4, very close to target
    let guardian_a = spawn_guardian(
        &mut app,
        parent,
        Vec2::new(72.0, -25.5), // almost at slot 4
        3,
        4,
        100.0,
        GuardianGridStep {
            step_x: 72.0,
            step_y: 26.0,
        },
    );
    // Guardian B already at slot 5
    let _guardian_b = spawn_guardian(
        &mut app,
        parent,
        Vec2::new(0.0, -26.0), // slot 5 position
        5,
        6,
        0.0, // stationary
        GuardianGridStep {
            step_x: 72.0,
            step_y: 26.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs(1));

    let slot_a = app.world().get::<GuardianSlot>(guardian_a).unwrap();
    assert_eq!(
        slot_a.0, 4,
        "guardian A should arrive at slot 4, got {}",
        slot_a.0
    );
    let target_a = app.world().get::<SlideTarget>(guardian_a).unwrap();
    // Clockwise from 4 is 5, but 5 is occupied by B — should skip to 6
    assert_eq!(
        target_a.0, 6,
        "guardian A should skip occupied slot 5 and target slot 6, got {}",
        target_a.0
    );
}

// Behavior 29: Single guardian with no occupied neighbors slides freely
#[test]
fn single_guardian_slides_freely_around_ring() {
    let mut app = test_app();
    let parent = spawn_guarded_parent(&mut app, Vec2::ZERO);
    let guardian = spawn_guardian(
        &mut app,
        parent,
        Vec2::new(-72.0, 26.0), // slot 0 position
        0,
        1,
        50.0,
        GuardianGridStep {
            step_x: 72.0,
            step_y: 26.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs(1));

    // With speed 50.0 and dt 1.0s, the guardian moves 50 units
    // It should have moved from slot 0 toward slot 1
    let pos = app.world().get::<Position2D>(guardian).unwrap();
    // The exact position depends on implementation, but it should have changed
    let original_pos = Vec2::new(-72.0, 26.0);
    let moved = (pos.0 - original_pos).length();
    assert!(
        moved > 0.0,
        "single guardian should have moved, but position is unchanged: {:?}",
        pos.0
    );
}

// Behavior 30: System only processes entities with GuardianCell marker
#[test]
fn system_only_processes_guardian_cell_entities() {
    let mut app = test_app();
    let parent = spawn_guarded_parent(&mut app, Vec2::ZERO);

    // Entity A: has GuardianCell marker — should be processed
    let guardian_a = spawn_guardian(
        &mut app,
        parent,
        Vec2::new(-72.0, 26.0),
        0,
        1,
        100.0,
        GuardianGridStep {
            step_x: 72.0,
            step_y: 26.0,
        },
    );

    // Entity B: has all components except GuardianCell marker — should NOT be processed
    let entity_b = app
        .world_mut()
        .spawn((
            Cell,
            // NO GuardianCell marker
            Spatial2D,
            Position2D(Vec2::new(-72.0, 26.0)),
            GuardianSlot(0),
            SlideTarget(1),
            GuardianSlideSpeed(100.0),
            GuardianGridStep {
                step_x: 72.0,
                step_y: 26.0,
            },
            ChildOf(parent),
        ))
        .id();

    tick_with_dt(&mut app, Duration::from_secs(1));

    // Entity A should have moved
    let pos_a = app.world().get::<Position2D>(guardian_a).unwrap();
    let original = Vec2::new(-72.0, 26.0);
    let moved_a = (pos_a.0 - original).length();
    assert!(
        moved_a > 0.0,
        "entity A (with GuardianCell) should have moved"
    );

    // Entity B should NOT have moved
    let pos_b = app.world().get::<Position2D>(entity_b).unwrap();
    assert!(
        (pos_b.0.x - (-72.0)).abs() < f32::EPSILON && (pos_b.0.y - 26.0).abs() < f32::EPSILON,
        "entity B (without GuardianCell) should not have moved, got {:?}",
        pos_b.0
    );
}

// Behavior 31: Guardian sliding respects parent position
#[test]
fn guardian_target_position_respects_parent_position() {
    let mut app = test_app();
    let parent = spawn_guarded_parent(&mut app, Vec2::new(100.0, 200.0));
    // Slot 0 offset is (-1.0, 1.0), so world pos = (100-72, 200+26) = (28.0, 226.0)
    let guardian = spawn_guardian(
        &mut app,
        parent,
        Vec2::new(28.0, 226.0), // slot 0 world pos
        0,
        1,      // target slot 1 → (100+0*72, 200+1*26) = (100.0, 226.0)
        1000.0, // high speed to ensure snap
        GuardianGridStep {
            step_x: 72.0,
            step_y: 26.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs(1));

    let pos = app.world().get::<Position2D>(guardian).unwrap();
    // Target slot 1 offset is (0.0, 1.0), world pos = (100.0+0.0*72.0, 200.0+1.0*26.0) = (100.0, 226.0)
    assert!(
        (pos.0.x - 100.0).abs() < 1.0 && (pos.0.y - 226.0).abs() < 1.0,
        "guardian slot 1 target should be (100.0, 226.0), got {:?}",
        pos.0
    );
}

// Behavior 31 edge case: parent at negative coordinates
#[test]
fn guardian_target_position_with_negative_parent() {
    let mut app = test_app();
    let parent = spawn_guarded_parent(&mut app, Vec2::new(-50.0, -100.0));
    // Slot 0 offset is (-1.0, 1.0), world pos = (-50-72, -100+26) = (-122.0, -74.0)
    let guardian = spawn_guardian(
        &mut app,
        parent,
        Vec2::new(-122.0, -74.0),
        0,
        1, // target slot 1 → (-50+0*72, -100+1*26) = (-50.0, -74.0)
        1000.0,
        GuardianGridStep {
            step_x: 72.0,
            step_y: 26.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs(1));

    let pos = app.world().get::<Position2D>(guardian).unwrap();
    assert!(
        (pos.0.x - (-50.0)).abs() < 1.0 && (pos.0.y - (-74.0)).abs() < 1.0,
        "guardian slot 1 target with negative parent should be (-50.0, -74.0), got {:?}",
        pos.0
    );
}
