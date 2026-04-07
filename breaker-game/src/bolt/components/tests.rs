use bevy::prelude::*;

use super::definitions::*;

// ── Bolt component tests (no #[require] — builder handles insertion) ────────

#[test]
fn bolt_does_not_auto_insert_spatial2d() {
    use rantzsoft_spatial2d::components::Spatial2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<Spatial2D>(entity).is_none(),
        "Bolt should NOT auto-insert Spatial2D (builder handles this)"
    );
}

#[test]
fn bolt_does_not_auto_insert_interpolate_transform2d() {
    use rantzsoft_spatial2d::components::InterpolateTransform2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<InterpolateTransform2D>(entity).is_none(),
        "Bolt should NOT auto-insert InterpolateTransform2D (builder handles this)"
    );
}

#[test]
fn bolt_does_not_auto_insert_velocity2d() {
    use rantzsoft_spatial2d::components::Velocity2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<Velocity2D>(entity).is_none(),
        "Bolt should NOT auto-insert Velocity2D (builder handles this)"
    );
}

#[test]
fn bolt_explicit_components_still_work() {
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();
    app.update();
    let velocity = app
        .world()
        .get::<Velocity2D>(entity)
        .expect("Velocity2D should be present when explicitly added");
    assert!(
        (velocity.0.x - 0.0).abs() < f32::EPSILON && (velocity.0.y - 400.0).abs() < f32::EPSILON,
        "explicit Velocity2D(0.0, 400.0) should be preserved, got {:?}",
        velocity.0
    );
    let position = app
        .world()
        .get::<Position2D>(entity)
        .expect("Position2D should be present when explicitly added");
    assert_eq!(
        position.0,
        Vec2::new(10.0, 20.0),
        "explicit Position2D(10.0, 20.0) should be preserved"
    );
}

#[test]
fn bolt_does_not_insert_cleanup_on_exit_run_state() {
    use rantzsoft_stateflow::CleanupOnExit;

    use crate::state::types::RunState;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<CleanupOnExit<RunState>>(entity).is_none(),
        "Bolt should NOT auto-insert CleanupOnExit<RunState>"
    );
}

#[test]
fn bolt_does_not_insert_cleanup_on_exit_node_state() {
    use rantzsoft_stateflow::CleanupOnExit;

    use crate::state::types::NodeState;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world()
            .get::<CleanupOnExit<NodeState>>(entity)
            .is_none(),
        "Bolt should NOT auto-insert CleanupOnExit<NodeState>"
    );
}

// ── BoltBaseDamage component tests ─────────────────────────────

// Behavior 16: BoltBaseDamage stores f32 damage value
#[test]
fn bolt_base_damage_stores_value() {
    let dmg = BoltBaseDamage(10.0);
    assert!(
        (dmg.0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage(10.0) should store 10.0, got {}",
        dmg.0
    );
}

#[test]
fn bolt_base_damage_zero_is_valid() {
    let dmg = BoltBaseDamage(0.0);
    assert!(
        dmg.0.abs() < f32::EPSILON,
        "BoltBaseDamage(0.0) should be valid, got {}",
        dmg.0
    );
}

// Behavior 17: BoltBaseDamage implements Debug
#[test]
fn bolt_base_damage_debug_contains_value() {
    let dmg = BoltBaseDamage(10.0);
    let debug_str = format!("{dmg:?}");
    assert!(
        !debug_str.is_empty(),
        "BoltBaseDamage Debug should produce non-empty string"
    );
    assert!(
        debug_str.contains("10"),
        "BoltBaseDamage Debug should contain '10', got: {debug_str}"
    );
}

// Behavior 18: BoltBaseDamage implements Clone and Copy
#[test]
fn bolt_base_damage_clone_and_copy() {
    let component = BoltBaseDamage(10.0);
    let a = component; // Copy
    let b = a; // Copy again (a is still valid because Copy)
    let c = component; // Copy (same as Clone for Copy types)
    assert!(
        (b.0 - 10.0).abs() < f32::EPSILON,
        "copied BoltBaseDamage should be 10.0"
    );
    assert!(
        (c.0 - 10.0).abs() < f32::EPSILON,
        "cloned BoltBaseDamage should be 10.0"
    );
}

// ── BoltDefinitionRef component tests ──────────────────────────

// Behavior 19: BoltDefinitionRef stores String reference to definition name
#[test]
fn bolt_definition_ref_stores_name() {
    let def_ref = BoltDefinitionRef("Bolt".to_string());
    assert_eq!(def_ref.0, "Bolt", "BoltDefinitionRef should store 'Bolt'");
}

#[test]
fn bolt_definition_ref_empty_string_is_valid() {
    let def_ref = BoltDefinitionRef(String::new());
    assert_eq!(
        def_ref.0, "",
        "BoltDefinitionRef with empty string should be constructible"
    );
}

// Behavior 20: BoltDefinitionRef implements Debug and Clone
#[test]
fn bolt_definition_ref_debug_and_clone() {
    let def_ref = BoltDefinitionRef("Heavy".to_string());
    let debug_str = format!("{def_ref:?}");
    assert!(
        debug_str.contains("Heavy"),
        "Debug should contain 'Heavy', got: {debug_str}"
    );
    let cloned = def_ref;
    assert_eq!(
        cloned.0, "Heavy",
        "cloned BoltDefinitionRef should be 'Heavy'"
    );
}

// ── BoltAngleSpread component tests ────────────────────────────

// Behavior 21: BoltAngleSpread stores f32 angle in radians
#[test]
fn bolt_angle_spread_stores_value() {
    let spread = BoltAngleSpread(0.524);
    assert!(
        (spread.0 - 0.524).abs() < f32::EPSILON,
        "BoltAngleSpread(0.524) should store 0.524, got {}",
        spread.0
    );
}

#[test]
fn bolt_angle_spread_zero_is_valid() {
    let spread = BoltAngleSpread(0.0);
    assert!(
        spread.0.abs() < f32::EPSILON,
        "BoltAngleSpread(0.0) should be valid, got {}",
        spread.0
    );
}

// Behavior 22: BoltAngleSpread implements Debug
#[test]
fn bolt_angle_spread_debug_contains_type_name() {
    let spread = BoltAngleSpread(0.524);
    let debug_str = format!("{spread:?}");
    assert!(
        !debug_str.is_empty(),
        "BoltAngleSpread Debug should produce non-empty string"
    );
    assert!(
        debug_str.contains("BoltAngleSpread"),
        "Debug should contain 'BoltAngleSpread', got: {debug_str}"
    );
}

// ── PrimaryBolt component tests ───────────────────────────────

#[test]
fn primary_bolt_is_zero_sized() {
    assert_eq!(
        std::mem::size_of::<PrimaryBolt>(),
        0,
        "PrimaryBolt should be a zero-sized type"
    );
}

#[test]
fn primary_bolt_has_debug() {
    let marker = PrimaryBolt;
    let debug_str = format!("{marker:?}");
    assert!(
        !debug_str.is_empty(),
        "PrimaryBolt should have a Debug impl"
    );
}

// ── CollisionLayers tests ──────────────────────────────────────

#[test]
fn bolt_collision_layers_have_correct_values() {
    use rantzsoft_physics2d::collision_layers::CollisionLayers;

    use crate::shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, WALL_LAYER};
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
        ))
        .id();
    app.update();
    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("Bolt should have CollisionLayers");
    assert_eq!(
        layers.membership, BOLT_LAYER,
        "Bolt membership should be BOLT_LAYER (0x{BOLT_LAYER:02X}), got 0x{:02X}",
        layers.membership
    );
    assert_eq!(
        layers.mask,
        CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
        "Bolt mask should be CELL|WALL|BREAKER (0x{:02X}), got 0x{:02X}",
        CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
        layers.mask
    );
}
