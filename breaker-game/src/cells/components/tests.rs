use bevy::prelude::*;

use super::*;

#[test]
fn cell_width_half_width() {
    let w = CellWidth::new(70.0);
    assert!((w.half_width() - 35.0).abs() < f32::EPSILON);
}

#[test]
fn cell_height_half_height() {
    let h = CellHeight::new(24.0);
    assert!((h.half_height() - 12.0).abs() < f32::EPSILON);
}

#[test]
fn cell_health_new_sets_current_and_max_to_given_hp() {
    let health = CellHealth::new(1.0);
    assert!((health.current - 1.0).abs() < f32::EPSILON);
    assert!((health.max - 1.0).abs() < f32::EPSILON);
    assert!(!health.is_destroyed());
}

#[test]
fn cell_health_not_destroyed_at_positive_hp() {
    let health = CellHealth::new(10.0);
    assert!(
        !health.is_destroyed(),
        "10.0 HP cell should not be destroyed"
    );
}

#[test]
fn cell_health_is_destroyed_at_exactly_zero() {
    let health = CellHealth {
        current: 0.0,
        max: 10.0,
    };
    assert!(
        health.is_destroyed(),
        "cell at exactly 0.0 HP should be destroyed"
    );
}

#[test]
fn cell_health_is_destroyed_when_negative_current() {
    // Edge case: overkill drives current below 0.0 — must still be destroyed.
    let health = CellHealth {
        current: -5.0,
        max: 10.0,
    };
    assert!(
        health.is_destroyed(),
        "cell with negative current HP (-5.0) should be destroyed (current <= 0.0)"
    );
}

#[test]
fn take_damage_10_destroys_10hp_cell() {
    let mut health = CellHealth::new(10.0);
    let destroyed = health.take_damage(10.0);
    assert!(
        destroyed,
        "take_damage(10.0) on 10.0-HP cell should return true"
    );
    assert!(
        health.current <= 0.0,
        "HP after lethal damage should be <= 0.0, got {}",
        health.current
    );
    assert!(health.is_destroyed());
}

#[test]
fn take_damage_15_on_10hp_cell_overkill() {
    // Overkill: damage exceeds remaining HP. Result must be <= 0 and destroyed=true.
    let mut health = CellHealth::new(10.0);
    let destroyed = health.take_damage(15.0);
    assert!(
        destroyed,
        "take_damage(15.0) on 10.0-HP cell should destroy it"
    );
    assert!(
        health.current <= 0.0,
        "overkill should leave HP <= 0.0, got {}",
        health.current
    );
}

#[test]
fn take_damage_10_on_30hp_cell_survives_with_20hp() {
    let mut health = CellHealth::new(30.0);
    let destroyed = health.take_damage(10.0);
    assert!(
        !destroyed,
        "take_damage(10.0) on 30.0-HP cell should not destroy it"
    );
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON,
        "30.0 HP - 10.0 damage = 20.0 HP remaining, got {}",
        health.current
    );
}

#[test]
fn take_damage_zero_on_already_dead_cell_returns_true() {
    // A cell already at 0 HP: take_damage(0.0) should still report destroyed.
    let mut health = CellHealth::new(10.0);
    health.take_damage(10.0); // kill it first
    let destroyed = health.take_damage(0.0);
    assert!(
        destroyed,
        "take_damage(0.0) on already-dead cell should return true (is_destroyed)"
    );
}

#[test]
fn take_damage_5_on_dead_cell_stays_destroyed() {
    // Dead cells should not go more negative than needed — and must remain destroyed.
    let mut health = CellHealth::new(10.0);
    health.take_damage(10.0); // kill it
    let destroyed = health.take_damage(5.0);
    assert!(destroyed, "take_damage on dead cell should return true");
    assert!(
        health.current <= 0.0,
        "dead cell should have HP <= 0.0 after further damage, got {}",
        health.current
    );
}

#[test]
fn cell_health_fraction_at_full_hp() {
    let health = CellHealth::new(4.0);
    assert!(
        (health.fraction() - 1.0).abs() < f32::EPSILON,
        "fraction at full HP should be 1.0"
    );
}

#[test]
fn cell_health_fraction_after_one_damage() {
    let mut health = CellHealth::new(4.0);
    health.take_damage(1.0);
    assert!(
        (health.fraction() - 0.75).abs() < 1e-5,
        "fraction after 1 damage on 4-HP cell should be 0.75, got {}",
        health.fraction()
    );
}

#[test]
fn cell_health_fraction_with_zero_max_returns_zero() {
    // Edge case: max == 0.0 guard prevents divide-by-zero.
    let health = CellHealth {
        current: 0.0,
        max: 0.0,
    };
    assert!(
        (health.fraction() - 0.0).abs() < f32::EPSILON,
        "fraction with max=0.0 should return 0.0"
    );
}

#[test]
fn cell_health_fraction_with_negative_current_clamped() {
    // Negative current (overkill) — fraction should not go below 0.0.
    let health = CellHealth {
        current: -5.0,
        max: 10.0,
    };
    assert!(
        health.fraction() <= 0.0,
        "fraction with negative current should be <= 0.0, got {}",
        health.fraction()
    );
}

// -- Cell #[require] tests ------------------------------------------------

#[test]
fn cell_require_inserts_spatial2d() {
    use rantzsoft_spatial2d::components::Spatial2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Cell).id();
    app.update();
    assert!(
        app.world().get::<Spatial2D>(entity).is_some(),
        "Cell should auto-insert Spatial2D via #[require]"
    );
}

#[test]
fn cell_require_inserts_cleanup_on_exit_node_state() {
    use rantzsoft_stateflow::CleanupOnExit;

    use crate::state::types::NodeState;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Cell).id();
    app.update();
    assert!(
        app.world()
            .get::<CleanupOnExit<NodeState>>(entity)
            .is_some(),
        "Cell should auto-insert CleanupOnExit<NodeState> via #[require]"
    );
}

#[test]
fn cell_require_does_not_insert_interpolate_transform2d() {
    use rantzsoft_spatial2d::components::InterpolateTransform2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Cell).id();
    app.update();
    assert!(
        app.world().get::<InterpolateTransform2D>(entity).is_none(),
        "Cell #[require] should NOT auto-insert InterpolateTransform2D (cells are static)"
    );
}

#[test]
fn cell_explicit_values_override_defaults() {
    use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial2D};
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app
        .world_mut()
        .spawn((
            Cell,
            Spatial2D,
            Position2D(Vec2::new(50.0, 100.0)),
            Scale2D { x: 70.0, y: 24.0 },
        ))
        .id();
    app.update();
    let position = app
        .world()
        .get::<Position2D>(entity)
        .expect("Position2D should be present");
    assert_eq!(
        position.0,
        Vec2::new(50.0, 100.0),
        "explicit Position2D(50.0, 100.0) should be preserved"
    );
    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("Scale2D should be present");
    assert!(
        (scale.x - 70.0).abs() < f32::EPSILON && (scale.y - 24.0).abs() < f32::EPSILON,
        "explicit Scale2D {{ x: 70.0, y: 24.0 }} should be preserved, got {{ x: {}, y: {} }}",
        scale.x,
        scale.y
    );
}

// -- CollisionLayers tests ------------------------------------------------

#[test]
fn cell_collision_layers_have_correct_values() {
    use rantzsoft_physics2d::collision_layers::CollisionLayers;

    use crate::shared::{BOLT_LAYER, CELL_LAYER};
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app
        .world_mut()
        .spawn((Cell, CollisionLayers::new(CELL_LAYER, BOLT_LAYER)))
        .id();
    app.update();
    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("Cell should have CollisionLayers");
    assert_eq!(
        layers.membership, CELL_LAYER,
        "Cell membership should be CELL_LAYER (0x{CELL_LAYER:02X}), got 0x{:02X}",
        layers.membership
    );
    assert_eq!(
        layers.mask, BOLT_LAYER,
        "Cell mask should be BOLT_LAYER (0x{BOLT_LAYER:02X}), got 0x{:02X}",
        layers.mask
    );
}

// -- take_damage(0.0) on living cell ------------------------------------------

#[test]
fn take_damage_zero_on_living_cell_returns_false() {
    let mut health = CellHealth::new(10.0);
    let destroyed = health.take_damage(0.0);

    assert!(
        !destroyed,
        "take_damage(0.0) on a living cell should return false"
    );
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "health.current should remain 10.0 after zero damage, got {}",
        health.current
    );
    assert!(
        !health.is_destroyed(),
        "cell should not be destroyed after zero damage"
    );

    // Edge case: barely alive cell with take_damage(0.0)
    let mut barely = CellHealth::new(0.001);
    let destroyed_barely = barely.take_damage(0.0);
    assert!(
        !destroyed_barely,
        "take_damage(0.0) on barely alive cell (0.001 HP) should return false"
    );
    assert!(
        (barely.current - 0.001).abs() < f32::EPSILON,
        "barely alive cell should remain at 0.001 HP, got {}",
        barely.current
    );
}
