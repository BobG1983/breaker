use super::*;
use crate::shared::{GameState, PlayingState};

// ── Behavior 24: process_piercing_beam damages all cells in beam path ──

#[test]
fn process_piercing_beam_damages_all_cells_in_beam_and_despawns() {
    let mut app = piercing_beam_damage_test_app();

    let cell_a = spawn_test_cell(&mut app, 0.0, 50.0);
    let cell_b = spawn_test_cell(&mut app, 0.0, 150.0);
    let cell_c = spawn_test_cell(&mut app, 0.0, 250.0);

    let request = app
        .world_mut()
        .spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 10.0,
            damage: 10.0,
        })
        .id();

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        3,
        "expected 3 DamageCell messages (one per cell), got {}",
        collector.0.len()
    );

    let damaged_cells: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
    assert!(damaged_cells.contains(&cell_a), "cell_a should be damaged");
    assert!(damaged_cells.contains(&cell_b), "cell_b should be damaged");
    assert!(damaged_cells.contains(&cell_c), "cell_c should be damaged");

    for msg in &collector.0 {
        assert!(
            (msg.damage - 10.0).abs() < f32::EPSILON,
            "each cell should receive damage 10.0"
        );
        assert!(msg.source_chip.is_none(), "source_chip should be None");
    }

    assert!(
        app.world().get_entity(request).is_err(),
        "PiercingBeamRequest entity should be despawned after processing"
    );
}

// ── Behavior 25: does not damage cells outside beam width ──

#[test]
fn process_piercing_beam_does_not_damage_cell_outside_beam_width() {
    let mut app = piercing_beam_damage_test_app();

    // Cell at (50, 100) — 50 units to the right of beam center
    // Beam half_width=10, so beam extends 10 units left/right
    spawn_test_cell(&mut app, 50.0, 100.0);

    let request = app
        .world_mut()
        .spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 10.0,
            damage: 10.0,
        })
        .id();

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "cell at (50, 100) is outside beam width (10+5 < 50) — no damage"
    );

    assert!(
        app.world().get_entity(request).is_err(),
        "request should be despawned"
    );
}

// ── Behavior 26: only targets cells on CELL_LAYER ──

#[test]
fn process_piercing_beam_only_targets_cell_layer() {
    let mut app = piercing_beam_damage_test_app();

    // Cell on CELL_LAYER
    let cell = spawn_test_cell(&mut app, 0.0, 50.0);

    // Entity on WALL_LAYER
    let wall_pos = Vec2::new(0.0, 100.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(WALL_LAYER, 0),
        Position2D(wall_pos),
        GlobalPosition2D(wall_pos),
        Spatial2D,
    ));

    // Entity on BOLT_LAYER
    let bolt_pos = Vec2::new(0.0, 150.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(BOLT_LAYER, 0),
        Position2D(bolt_pos),
        GlobalPosition2D(bolt_pos),
        Spatial2D,
    ));

    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: Vec2::new(0.0, 1.0),
        length: 300.0,
        half_width: 10.0,
        damage: 10.0,
    });

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "only CELL_LAYER entity should be damaged, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);
}

#[test]
fn process_piercing_beam_targets_entity_with_combined_cell_layer() {
    let mut app = piercing_beam_damage_test_app();

    // Entity with CELL_LAYER | WALL_LAYER
    let pos = Vec2::new(0.0, 50.0);
    let combined = app
        .world_mut()
        .spawn((
            Cell,
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(CELL_LAYER | WALL_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id();

    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: Vec2::new(0.0, 1.0),
        length: 300.0,
        half_width: 10.0,
        damage: 10.0,
    });

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "entity with CELL_LAYER in combined mask should be damaged"
    );
    assert_eq!(collector.0[0].cell, combined);
}

// ── Behavior 27: no cells in beam path — despawns without damage ──

#[test]
fn process_piercing_beam_no_cells_despawns_without_damage() {
    let mut app = piercing_beam_damage_test_app();

    let request = app
        .world_mut()
        .spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 10.0,
            damage: 10.0,
        })
        .id();

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "no cells — zero DamageCell messages"
    );

    assert!(
        app.world().get_entity(request).is_err(),
        "request should be despawned even with no cells"
    );
}

#[test]
fn process_piercing_beam_cells_outside_beam_rectangle_no_damage() {
    let mut app = piercing_beam_damage_test_app();

    // Cell far to the right — outside beam
    spawn_test_cell(&mut app, 200.0, 100.0);

    let request = app
        .world_mut()
        .spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 10.0,
            damage: 10.0,
        })
        .id();

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "cell outside beam rectangle should not be damaged"
    );

    assert!(
        app.world().get_entity(request).is_err(),
        "request should be despawned"
    );
}

// ── Behavior 28: each cell damaged at most once per beam ──

#[test]
fn process_piercing_beam_damages_each_cell_at_most_once() {
    let mut app = piercing_beam_damage_test_app();

    // One cell with large AABB that overlaps beam
    let pos = Vec2::new(0.0, 50.0);
    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Aabb2D::new(Vec2::ZERO, Vec2::new(20.0, 20.0)),
            CollisionLayers::new(CELL_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id();

    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: Vec2::new(0.0, 1.0),
        length: 300.0,
        half_width: 20.0,
        damage: 10.0,
    });

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "cell should be damaged exactly once, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);
}

// ── Behavior 29: Multiple requests processed independently ──

#[test]
fn multiple_piercing_beam_requests_processed_independently() {
    let mut app = piercing_beam_damage_test_app();

    // One cell in both beams' paths
    let cell = spawn_test_cell(&mut app, 0.0, 50.0);

    let req1 = app
        .world_mut()
        .spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 10.0,
            damage: 10.0,
        })
        .id();

    let req2 = app
        .world_mut()
        .spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 10.0,
            damage: 20.0,
        })
        .id();

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "two beams hitting same cell should produce 2 DamageCell messages, got {}",
        collector.0.len()
    );

    for msg in &collector.0 {
        assert_eq!(msg.cell, cell, "both messages should target the same cell");
    }

    let mut damages: Vec<f32> = collector.0.iter().map(|m| m.damage).collect();
    damages.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(
        (damages[0] - 10.0).abs() < f32::EPSILON,
        "expected damage 10.0"
    );
    assert!(
        (damages[1] - 20.0).abs() < f32::EPSILON,
        "expected damage 20.0"
    );

    assert!(
        app.world().get_entity(req1).is_err(),
        "first request should be despawned"
    );
    assert!(
        app.world().get_entity(req2).is_err(),
        "second request should be despawned"
    );
}

// ── Behavior 30: diagonal beam ──

#[test]
fn process_piercing_beam_handles_diagonal_beam() {
    let mut app = piercing_beam_damage_test_app();

    // Cell on diagonal path at (100, 100)
    let cell_on_path = spawn_test_cell(&mut app, 100.0, 100.0);
    // Cell off diagonal path at (100, 0)
    let cell_off_path = spawn_test_cell(&mut app, 100.0, 0.0);

    let dir = Vec2::new(1.0, 1.0).normalize();
    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: dir,
        length: 400.0,
        half_width: 15.0,
        damage: 10.0,
    });

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    // Cell at (100, 100) is on the 45-degree diagonal path and should be hit
    let cells_hit: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
    assert!(
        cells_hit.contains(&cell_on_path),
        "cell at (100, 100) should be hit by diagonal beam"
    );
    assert!(
        !cells_hit.contains(&cell_off_path),
        "cell off diagonal path should not be damaged"
    );
}

// ── Behavior 31: register() wires the process system ──

#[test]
fn register_wires_process_piercing_beam_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_plugins(RantzPhysics2dPlugin);
    app.init_state::<GameState>();
    app.add_sub_state::<PlayingState>();
    app.add_message::<DamageCell>();
    app.insert_resource(DamageCellCollector::default());
    app.insert_resource(PlayfieldConfig::default());
    app.add_systems(Update, collect_damage_cells);

    register(&mut app);

    // Transition to PlayingState::Active so the run_if guard passes
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Spawn a request — if register() wires the system, it should be processed
    let request = app
        .world_mut()
        .spawn(PiercingBeamRequest {
            origin: Vec2::ZERO,
            direction: Vec2::Y,
            length: 300.0,
            half_width: 10.0,
            damage: 10.0,
        })
        .id();

    tick(&mut app);

    // The request should be despawned after processing
    assert!(
        app.world().get_entity(request).is_err(),
        "register() should wire process_piercing_beam — request should be despawned after tick"
    );
}

// ── Damage scaling: end-to-end piercing beam damage includes EDM ──

#[test]
fn piercing_beam_end_to_end_damage_includes_effective_damage_multiplier() {
    // End-to-end test: fire() pre-computes damage with EDM, process sends DamageCell
    let mut app = piercing_beam_damage_test_app();

    let cell = spawn_test_cell(&mut app, 0.0, 50.0);

    // Source entity with EDM
    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity2D(Vec2::new(0.0, 400.0)),
            crate::effect::EffectiveDamageMultiplier(2.0),
        ))
        .id();

    // fire() should read EDM and pre-compute scaled damage
    fire(entity, 1.5, 20.0, "", app.world_mut());

    // Tick to run process_piercing_beam
    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected 1 DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);

    // damage = BASE_BOLT_DAMAGE * damage_mult * EDM = 10.0 * 1.5 * 2.0 = 30.0
    let expected_damage = BASE_BOLT_DAMAGE * 1.5 * 2.0;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "end-to-end damage should be {} (10.0 * 1.5 * 2.0), got {}",
        expected_damage,
        collector.0[0].damage
    );
    assert!(
        collector.0[0].source_chip.is_none(),
        "source_chip should be None"
    );
}

// -- Section G: EffectSourceChip attribution tests ───────────────────

use crate::effect::core::EffectSourceChip;

#[test]
fn fire_stores_effect_source_chip_with_non_empty_chip_name() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 2.0, 10.0, "laser", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0,
        Some("laser".to_string()),
        "spawned PiercingBeamRequest should have EffectSourceChip(Some(\"laser\"))"
    );
}

#[test]
fn fire_stores_effect_source_chip_none_with_empty_chip_name() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 2.0, 10.0, "", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0, None,
        "empty source_chip should produce EffectSourceChip(None)"
    );
}

#[test]
fn process_piercing_beam_populates_source_chip_from_effect_source_chip() {
    let mut app = piercing_beam_damage_test_app();

    let _cell_a = spawn_test_cell(&mut app, 0.0, 50.0);
    let _cell_b = spawn_test_cell(&mut app, 0.0, 150.0);

    app.world_mut().spawn((
        PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 10.0,
            damage: 10.0,
        },
        EffectSourceChip(Some("laser".to_string())),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "expected 2 DamageCell messages, got {}",
        collector.0.len()
    );

    for msg in &collector.0 {
        assert_eq!(
            msg.source_chip,
            Some("laser".to_string()),
            "DamageCell should have source_chip from EffectSourceChip"
        );
    }
}

#[test]
fn process_piercing_beam_source_chip_none_when_effect_source_chip_none() {
    let mut app = piercing_beam_damage_test_app();

    spawn_test_cell(&mut app, 0.0, 50.0);

    app.world_mut().spawn((
        PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 10.0,
            damage: 10.0,
        },
        EffectSourceChip(None),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].source_chip, None,
        "EffectSourceChip(None) should produce source_chip None"
    );
}

#[test]
fn process_piercing_beam_defaults_to_none_when_no_effect_source_chip() {
    let mut app = piercing_beam_damage_test_app();

    spawn_test_cell(&mut app, 0.0, 50.0);

    // No EffectSourceChip on request
    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: Vec2::new(0.0, 1.0),
        length: 300.0,
        half_width: 10.0,
        damage: 10.0,
    });

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].source_chip, None,
        "missing EffectSourceChip should default to source_chip None"
    );
}
