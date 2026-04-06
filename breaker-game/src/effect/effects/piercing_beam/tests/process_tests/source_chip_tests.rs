use crate::effect::{core::EffectSourceChip, effects::piercing_beam::tests::helpers::*};

#[test]
fn fire_stores_effect_source_chip_with_non_empty_chip_name() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((Position2D(Vec2::ZERO), Velocity2D(Vec2::new(0.0, 400.0))))
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
        .spawn((Position2D(Vec2::ZERO), Velocity2D(Vec2::new(0.0, 400.0))))
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
