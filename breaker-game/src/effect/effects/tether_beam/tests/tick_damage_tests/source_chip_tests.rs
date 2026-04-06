//! Tests for `EffectSourceChip` attribution on `tick_tether_beam` damage messages.

use super::super::helpers::*;
use crate::effect::core::EffectSourceChip;

#[test]
fn tick_tether_beam_populates_source_chip_from_effect_source_chip() {
    let mut app = damage_test_app();

    let bolt_a = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(0.0, 0.0)),
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Spatial2D,
        ))
        .id();
    let bolt_b = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(100.0, 0.0)),
            GlobalPosition2D(Vec2::new(100.0, 0.0)),
            Spatial2D,
        ))
        .id();
    let _beam = app
        .world_mut()
        .spawn((
            TetherBeamComponent {
                bolt_a,
                bolt_b,
                damage_mult: 2.0,
                effective_damage_multiplier: 1.0,
                base_damage: DEFAULT_BOLT_BASE_DAMAGE,
            },
            EffectSourceChip(Some("tether".to_string())),
            CleanupOnExit::<NodeState>::default(),
        ))
        .id();
    app.world_mut().entity_mut(bolt_a).insert(TetherBoltMarker);
    app.world_mut().entity_mut(bolt_b).insert(TetherBoltMarker);

    let cell = spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected 1 DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);
    assert_eq!(
        collector.0[0].source_chip,
        Some("tether".to_string()),
        "DamageCell should have source_chip from beam's EffectSourceChip"
    );
}

#[test]
fn tick_tether_beam_source_chip_none_when_effect_source_chip_none() {
    let mut app = damage_test_app();

    let bolt_a = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(0.0, 0.0)),
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Spatial2D,
        ))
        .id();
    let bolt_b = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(100.0, 0.0)),
            GlobalPosition2D(Vec2::new(100.0, 0.0)),
            Spatial2D,
        ))
        .id();
    let _beam = app
        .world_mut()
        .spawn((
            TetherBeamComponent {
                bolt_a,
                bolt_b,
                damage_mult: 2.0,
                effective_damage_multiplier: 1.0,
                base_damage: DEFAULT_BOLT_BASE_DAMAGE,
            },
            EffectSourceChip(None),
            CleanupOnExit::<NodeState>::default(),
        ))
        .id();
    app.world_mut().entity_mut(bolt_a).insert(TetherBoltMarker);
    app.world_mut().entity_mut(bolt_b).insert(TetherBoltMarker);

    spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].source_chip, None,
        "EffectSourceChip(None) should produce source_chip None"
    );
}

#[test]
fn tick_tether_beam_defaults_to_none_when_no_effect_source_chip() {
    let mut app = damage_test_app();

    let bolt_a = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(0.0, 0.0)),
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Spatial2D,
        ))
        .id();
    let bolt_b = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(100.0, 0.0)),
            GlobalPosition2D(Vec2::new(100.0, 0.0)),
            Spatial2D,
        ))
        .id();
    let _beam = app
        .world_mut()
        .spawn((
            TetherBeamComponent {
                bolt_a,
                bolt_b,
                damage_mult: 2.0,
                effective_damage_multiplier: 1.0,
                base_damage: DEFAULT_BOLT_BASE_DAMAGE,
            },
            CleanupOnExit::<NodeState>::default(),
        ))
        .id();
    app.world_mut().entity_mut(bolt_a).insert(TetherBoltMarker);
    app.world_mut().entity_mut(bolt_b).insert(TetherBoltMarker);

    spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].source_chip, None,
        "missing EffectSourceChip should default to source_chip None"
    );
}

#[test]
fn multiple_tether_beams_with_different_source_chips_produce_correctly_attributed_damage() {
    let mut app = damage_test_app();

    // Beam A: horizontal at y=0
    let alpha_left = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(0.0, 0.0)),
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Spatial2D,
        ))
        .id();
    let alpha_right = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(100.0, 0.0)),
            GlobalPosition2D(Vec2::new(100.0, 0.0)),
            Spatial2D,
        ))
        .id();
    let _beam_a = app
        .world_mut()
        .spawn((
            TetherBeamComponent {
                bolt_a: alpha_left,
                bolt_b: alpha_right,
                damage_mult: 1.0,
                effective_damage_multiplier: 1.0,
                base_damage: DEFAULT_BOLT_BASE_DAMAGE,
            },
            EffectSourceChip(Some("alpha".to_string())),
            CleanupOnExit::<NodeState>::default(),
        ))
        .id();
    app.world_mut()
        .entity_mut(alpha_left)
        .insert(TetherBoltMarker);
    app.world_mut()
        .entity_mut(alpha_right)
        .insert(TetherBoltMarker);

    // Beam B: horizontal at y=200
    let beta_left = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(0.0, 200.0)),
            GlobalPosition2D(Vec2::new(0.0, 200.0)),
            Spatial2D,
        ))
        .id();
    let beta_right = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(100.0, 200.0)),
            GlobalPosition2D(Vec2::new(100.0, 200.0)),
            Spatial2D,
        ))
        .id();
    let _beam_b = app
        .world_mut()
        .spawn((
            TetherBeamComponent {
                bolt_a: beta_left,
                bolt_b: beta_right,
                damage_mult: 1.0,
                effective_damage_multiplier: 1.0,
                base_damage: DEFAULT_BOLT_BASE_DAMAGE,
            },
            EffectSourceChip(Some("beta".to_string())),
            CleanupOnExit::<NodeState>::default(),
        ))
        .id();
    app.world_mut()
        .entity_mut(beta_left)
        .insert(TetherBoltMarker);
    app.world_mut()
        .entity_mut(beta_right)
        .insert(TetherBoltMarker);

    let cell_a = spawn_test_cell(&mut app, 50.0, 0.0);
    let cell_b = spawn_test_cell(&mut app, 50.0, 200.0);

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
        "cell near beam A should have source_chip alpha"
    );

    let msg_b = collector.0.iter().find(|m| m.cell == cell_b).unwrap();
    assert_eq!(
        msg_b.source_chip,
        Some("beta".to_string()),
        "cell near beam B should have source_chip beta"
    );
}
