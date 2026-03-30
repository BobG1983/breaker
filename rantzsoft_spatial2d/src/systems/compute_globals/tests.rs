use super::system::*;
use crate::{
    components::{
        GlobalPosition2D, GlobalRotation2D, GlobalScale2D, Position2D, Rotation2D, Scale2D,
        Spatial2D,
    },
    propagation::{PositionPropagation, RotationPropagation, ScalePropagation},
};
use bevy::prelude::*;

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

// ── Deep hierarchy: 5-level Relative chain accumulates correctly ──

#[test]
fn five_level_relative_hierarchy_accumulates_position() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, compute_globals);

    let root = app
        .world_mut()
        .spawn((Spatial2D, Position2D(Vec2::new(100.0, 0.0))))
        .id();

    let a = app
        .world_mut()
        .spawn((
            ChildOf(root),
            Spatial2D,
            Position2D(Vec2::new(10.0, 0.0)),
            PositionPropagation::Relative,
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            ChildOf(a),
            Spatial2D,
            Position2D(Vec2::new(10.0, 0.0)),
            PositionPropagation::Relative,
        ))
        .id();

    let c = app
        .world_mut()
        .spawn((
            ChildOf(b),
            Spatial2D,
            Position2D(Vec2::new(10.0, 0.0)),
            PositionPropagation::Relative,
        ))
        .id();

    let d = app
        .world_mut()
        .spawn((
            ChildOf(c),
            Spatial2D,
            Position2D(Vec2::new(10.0, 0.0)),
            PositionPropagation::Relative,
        ))
        .id();

    tick(&mut app);

    let root_pos = app.world().get::<GlobalPosition2D>(root).unwrap();
    assert_eq!(
        root_pos.0,
        Vec2::new(100.0, 0.0),
        "root GlobalPosition2D should be (100, 0)"
    );

    let a_pos = app.world().get::<GlobalPosition2D>(a).unwrap();
    assert_eq!(
        a_pos.0,
        Vec2::new(110.0, 0.0),
        "A GlobalPosition2D should be root(100) + local(10) = 110"
    );

    let b_pos = app.world().get::<GlobalPosition2D>(b).unwrap();
    assert_eq!(
        b_pos.0,
        Vec2::new(120.0, 0.0),
        "B GlobalPosition2D should be A(110) + local(10) = 120"
    );

    let c_pos = app.world().get::<GlobalPosition2D>(c).unwrap();
    assert_eq!(
        c_pos.0,
        Vec2::new(130.0, 0.0),
        "C GlobalPosition2D should be B(120) + local(10) = 130"
    );

    let d_pos = app.world().get::<GlobalPosition2D>(d).unwrap();
    assert_eq!(
        d_pos.0,
        Vec2::new(140.0, 0.0),
        "D GlobalPosition2D should be C(130) + local(10) = 140"
    );
}

// ── Deep hierarchy: mixed Relative and Absolute propagation ──

#[test]
fn mixed_relative_absolute_deep_hierarchy_propagates_correctly() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, compute_globals);

    // root: Position2D(100.0, 0.0)
    let root = app
        .world_mut()
        .spawn((Spatial2D, Position2D(Vec2::new(100.0, 0.0))))
        .id();

    // A: Relative child of root, Position2D(50.0, 0.0)
    let a = app
        .world_mut()
        .spawn((
            ChildOf(root),
            Spatial2D,
            Position2D(Vec2::new(50.0, 0.0)),
            PositionPropagation::Relative,
        ))
        .id();

    // B: Absolute child of A, Position2D(20.0, 0.0) — ignores parent
    let b = app
        .world_mut()
        .spawn((
            ChildOf(a),
            Spatial2D,
            Position2D(Vec2::new(20.0, 0.0)),
            PositionPropagation::Absolute,
        ))
        .id();

    // C: Relative child of B, Position2D(5.0, 0.0)
    let c = app
        .world_mut()
        .spawn((
            ChildOf(b),
            Spatial2D,
            Position2D(Vec2::new(5.0, 0.0)),
            PositionPropagation::Relative,
        ))
        .id();

    tick(&mut app);

    let root_pos = app.world().get::<GlobalPosition2D>(root).unwrap();
    assert_eq!(
        root_pos.0,
        Vec2::new(100.0, 0.0),
        "root GlobalPosition2D should be (100, 0)"
    );

    let a_pos = app.world().get::<GlobalPosition2D>(a).unwrap();
    assert_eq!(
        a_pos.0,
        Vec2::new(150.0, 0.0),
        "A GlobalPosition2D should be root(100) + local(50) = 150"
    );

    let b_pos = app.world().get::<GlobalPosition2D>(b).unwrap();
    assert_eq!(
        b_pos.0,
        Vec2::new(20.0, 0.0),
        "B GlobalPosition2D should be (20, 0) — Absolute ignores parent"
    );

    let c_pos = app.world().get::<GlobalPosition2D>(c).unwrap();
    assert_eq!(
        c_pos.0,
        Vec2::new(25.0, 0.0),
        "C GlobalPosition2D should be B_global(20) + local(5) = 25"
    );
}

// ── Deep hierarchy: rotation + scale propagation through 3 levels ──

#[test]
fn deep_rotation_and_scale_hierarchy_accumulates() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, compute_globals);

    // root: 90 degrees, scale (2, 2)
    let root = app
        .world_mut()
        .spawn((
            Spatial2D,
            Rotation2D::from_degrees(90.0),
            Scale2D { x: 2.0, y: 2.0 },
        ))
        .id();

    // A: Relative child, 45 degrees, scale (3, 3)
    let a = app
        .world_mut()
        .spawn((
            ChildOf(root),
            Spatial2D,
            Rotation2D::from_degrees(45.0),
            Scale2D { x: 3.0, y: 3.0 },
            RotationPropagation::Relative,
            ScalePropagation::Relative,
        ))
        .id();

    // B: Relative child, 10 degrees, scale (0.5, 0.5)
    let b = app
        .world_mut()
        .spawn((
            ChildOf(a),
            Spatial2D,
            Rotation2D::from_degrees(10.0),
            Scale2D { x: 0.5, y: 0.5 },
            RotationPropagation::Relative,
            ScalePropagation::Relative,
        ))
        .id();

    tick(&mut app);

    // Root: GlobalRotation = 90 degrees, GlobalScale = (2, 2)
    let root_rot = app.world().get::<GlobalRotation2D>(root).unwrap();
    let root_scale = app.world().get::<GlobalScale2D>(root).unwrap();
    assert!(
        (root_rot.0.as_degrees() - 90.0).abs() < 0.1,
        "root GlobalRotation2D should be ~90 degrees, got {}",
        root_rot.0.as_degrees()
    );
    assert!(
        (root_scale.x - 2.0).abs() < f32::EPSILON,
        "root GlobalScale2D.x should be 2.0, got {}",
        root_scale.x
    );
    assert!(
        (root_scale.y - 2.0).abs() < f32::EPSILON,
        "root GlobalScale2D.y should be 2.0, got {}",
        root_scale.y
    );

    // A: GlobalRotation = 90 + 45 = 135 degrees, GlobalScale = 2*3 = (6, 6)
    let a_rot = app.world().get::<GlobalRotation2D>(a).unwrap();
    let a_scale = app.world().get::<GlobalScale2D>(a).unwrap();
    let expected_rot_a = 135.0_f32.to_radians();
    assert!(
        (a_rot.0.as_radians() - expected_rot_a).abs() < 1e-4,
        "A GlobalRotation2D should be ~135 degrees ({} rad), got {} rad",
        expected_rot_a,
        a_rot.0.as_radians()
    );
    assert!(
        (a_scale.x - 6.0).abs() < f32::EPSILON,
        "A GlobalScale2D.x should be 2*3=6, got {}",
        a_scale.x
    );
    assert!(
        (a_scale.y - 6.0).abs() < f32::EPSILON,
        "A GlobalScale2D.y should be 2*3=6, got {}",
        a_scale.y
    );

    // B: GlobalRotation = 90 + 45 + 10 = 145 degrees, GlobalScale = 6*0.5 = (3, 3)
    let b_rot = app.world().get::<GlobalRotation2D>(b).unwrap();
    let b_scale = app.world().get::<GlobalScale2D>(b).unwrap();
    let expected_rot_b = 145.0_f32.to_radians();
    assert!(
        (b_rot.0.as_radians() - expected_rot_b).abs() < 1e-4,
        "B GlobalRotation2D should be ~145 degrees ({} rad), got {} rad",
        expected_rot_b,
        b_rot.0.as_radians()
    );
    assert!(
        (b_scale.x - 3.0).abs() < f32::EPSILON,
        "B GlobalScale2D.x should be 6*0.5=3, got {}",
        b_scale.x
    );
    assert!(
        (b_scale.y - 3.0).abs() < f32::EPSILON,
        "B GlobalScale2D.y should be 6*0.5=3, got {}",
        b_scale.y
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
