//! Tests for `EffectSourceChip` attribution on shockwave damage and
//! `Position2D`-based positioning for `apply_shockwave_damage`.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::effect::{core::EffectSourceChip, effects::shockwave::tests::helpers::*};

// -- Section C: EffectSourceChip attribution tests ───────────────────

#[test]
fn apply_shockwave_damage_populates_source_chip_from_effect_source_chip() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 20.0, 0.0);

    app.world_mut().spawn((
        ShockwaveSource,
        ShockwaveRadius(35.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        EffectSourceChip(Some("seismic".to_string())),
        Position2D(Vec2::new(0.0, 0.0)),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1, "expected one DamageCell message");
    assert_eq!(collector.0[0].cell, cell);
    assert_eq!(
        collector.0[0].source_chip,
        Some("seismic".to_string()),
        "DamageCell should have source_chip from EffectSourceChip"
    );
}

#[test]
fn apply_shockwave_damage_source_chip_none_when_effect_source_chip_none() {
    let mut app = damage_test_app();

    spawn_test_cell(&mut app, 20.0, 0.0);

    app.world_mut().spawn((
        ShockwaveSource,
        ShockwaveRadius(35.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        EffectSourceChip(None),
        Position2D(Vec2::new(0.0, 0.0)),
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
fn apply_shockwave_damage_defaults_to_none_when_no_effect_source_chip_component() {
    let mut app = damage_test_app();

    spawn_test_cell(&mut app, 20.0, 0.0);

    // No EffectSourceChip component on shockwave
    app.world_mut().spawn((
        ShockwaveSource,
        ShockwaveRadius(35.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        Position2D(Vec2::new(0.0, 0.0)),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].source_chip, None,
        "missing EffectSourceChip should default to source_chip None"
    );
}

#[test]
fn multiple_shockwaves_with_different_source_chips_produce_correctly_attributed_damage() {
    let mut app = damage_test_app();

    let cell_a = spawn_test_cell(&mut app, 15.0, 0.0);
    let cell_b = spawn_test_cell(&mut app, 90.0, 0.0);

    app.world_mut().spawn((
        ShockwaveSource,
        ShockwaveRadius(25.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        EffectSourceChip(Some("alpha".to_string())),
        Position2D(Vec2::new(0.0, 0.0)),
    ));

    app.world_mut().spawn((
        ShockwaveSource,
        ShockwaveRadius(25.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        EffectSourceChip(Some("beta".to_string())),
        Position2D(Vec2::new(100.0, 0.0)),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "expected 2 DamageCell messages, got {}",
        collector.0.len()
    );

    let msg_a = collector.0.iter().find(|m| m.cell == cell_a).unwrap();
    assert_eq!(
        msg_a.source_chip,
        Some("alpha".to_string()),
        "cell near shockwave A should have source_chip alpha"
    );

    let msg_b = collector.0.iter().find(|m| m.cell == cell_b).unwrap();
    assert_eq!(
        msg_b.source_chip,
        Some("beta".to_string()),
        "cell near shockwave B should have source_chip beta"
    );
}

// ── Behavior: apply_shockwave_damage uses Position2D not Transform when both present ──

#[test]
fn apply_shockwave_damage_uses_position2d_not_transform_when_both_present() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);

    // Shockwave at Position2D origin (0,0), but Transform at (500,500) — divergent.
    // If the system reads Position2D, cell at (30,0) is within radius 35.
    // If the system incorrectly reads Transform, cell would be ~530 units away — outside radius.
    app.world_mut().spawn((
        ShockwaveSource,
        ShockwaveRadius(35.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        Position2D(Vec2::new(0.0, 0.0)),
        Transform::from_xyz(500.0, 500.0, 0.0),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "cell at (30,0) should be within radius 35 of Position2D (0,0), got {} messages",
        collector.0.len()
    );
    assert_eq!(
        collector.0[0].cell, cell,
        "DamageCell should target the cell within Position2D-based radius"
    );
}
